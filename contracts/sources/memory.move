module omo::memory {
    use trie::hash_value::{HashValue};
    use signed_integer::bits::Bits;
    use StarcoinFramework::Vector::{length};
    use signed_integer::bits;
    use trie::trie;
    use StarcoinFramework::Vector;
    use StarcoinFramework::BCS;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Option;
    use trie::rlp;
    use trie::rlp_stream;
    use StarcoinFramework::Signer;

    const MISSING_REG_DATA: u64 = 200;
    const MEM_ACCESS_MUTST_BE_ALIGNED_TO_4BYTES: u64 = 401;
    const BITS_LEN_OUT_OF_BOUND: u64 = 601;
    const ELEM_NOT_FOUND: u64 = 701;

    const REG_FAKE_ADDRESS_START: u64 = 0xffffffff + 1;


    struct MemoryStorage has key, store {
        data: Option::Option<trie::TrieDB>,
    }

    const REG_KEY: vector<u8> = vector[0,0,0,0];
    struct Register has store, drop, copy {
        id: u64,
        value: u64
    }

    /// A flash struct whose lifetime only constraints to one transaction.
    /// It has no copy,drop, store, key
    struct Memory {
        storage_handle: address,
        data: trie::TrieDB,
        registers: vector<Register>,
        root: HashValue,
    }

    public fun create(signer: &signer) {
        move_to(signer, MemoryStorage {
            data: Option::some(trie::new()),
        })
    }
    public fun create_if_not_exists(signer: &signer) {
        if (!exists<MemoryStorage>(Signer::address_of(signer))) {
            create(signer)
        }
    }

    public fun batch_add_trie_data(mem_addr: address, data: vector<vector<u8>>) acquires MemoryStorage {
        let db = borrow_global_mut<MemoryStorage>(mem_addr);
        {
            let db = Option::borrow_mut(&mut db.data);
            let i = Vector::length(&data);
            while (i != 0) {
                trie::add_raw_node(db, Vector::pop_back(&mut data));
                i = i - 1;
            };
        };
    }

    public fun get_mem(memory_addr: address, state_root: HashValue): Memory acquires MemoryStorage {
         let mem = Memory {
             storage_handle: memory_addr,
             data: Option::extract(&mut borrow_global_mut<MemoryStorage>(memory_addr).data),
             registers: Vector::empty(),
             root: state_root,
         };
        recover_registers(&mut mem);
        StarcoinFramework::Debug::print(&mem.registers);
        mem
    }
    fun recover_registers(mem: &mut Memory) {
        let v = trie::get(&mem.data, mem.root, &REG_KEY);
        if (Option::is_none(&v)) {
            abort Errors::invalid_state(MISSING_REG_DATA)
        } else {
            let register_data = Option::destroy_some(v);

            let r = rlp::new(register_data);
            let vl = rlp::as_valuelist(&r);
            let i =0;
            while (i < Vector::length(&vl)) {
                let d = from_be_bytes(Vector::borrow(&vl, i));
                let id = d >> 32;
                let v = (d << 32) >> 32;
                set_register(mem, id, v);
                i = i + 1;
            };
        }
    }

    fun serialize_registers(regs: &vector<Register>): vector<u8> {
        StarcoinFramework::Debug::print(regs);
        let ser = rlp_stream::new_list(length(regs));
        let i = 0;
        while (i < length(regs)) {
            let reg = Vector::borrow(regs, i);

            let encoded_reg = {
                let encoded_reg = (reg.id << 32) + reg.value;
                let encoded = BCS::to_bytes(&encoded_reg);
                Vector::reverse(&mut encoded);
                encoded
            };
            rlp_stream::append(&mut ser, encoded_reg);
            i = i + 1;
        };
        rlp_stream::out(ser)
    }

    public fun return_mem(mem: Memory): HashValue
    acquires MemoryStorage {
        let Memory {data, storage_handle, registers, root} = mem;
        let ser_regs = serialize_registers(&registers);
        StarcoinFramework::Debug::print(&ser_regs);
        let root = trie::update(&mut data, root, REG_KEY, ser_regs);
        Option::fill(&mut borrow_global_mut<MemoryStorage>(storage_handle).data, data);
        root
    }


    public fun borrow_db_mut(mem: &mut Memory): &mut trie::TrieDB {
        &mut mem.data
    }

    /// Set register, and make sure the vec is sorted by register id
    fun set_register(mem: &mut Memory, id: u64, v: u64) {
        let temp = Vector::empty();
        while (!Vector::is_empty(&mem.registers)){
            let e = Vector::pop_back(&mut mem.registers);
            if (e.id < id) {
                Vector::push_back(&mut mem.registers, e);
                break
            } else if (e.id == id) {
                break
            } else {
                Vector::push_back(&mut temp, e);
            }
        };

        Vector::push_back(&mut mem.registers, Register {id, value: v});
        if (!Vector::is_empty(&temp)) {
            Vector::reverse(&mut temp);
            Vector::append(&mut mem.registers, temp);
        }
    }
    public fun get_register(mem: &Memory, id: u64): u64 {
        let i = 0;
        while (i < length(&mem.registers)) {
            let elem = Vector::borrow(&mem.registers, i);
            if (elem.id == id) {
                return elem.value
            } else if (elem.id > id) {
                // default to 0, if not found
                return 0
            };
            i = i+1;
        };
        // default to 0, if not found
        0
    }



    /// Read memory in four-bytes and convert it to u32 as big-endian representation.
    public fun read_memory(mem: &Memory, state_hash: HashValue, addr: u64): u64 {
        assert!(addr & 3 == 0, Errors::invalid_argument(MEM_ACCESS_MUTST_BE_ALIGNED_TO_4BYTES));
        let key = to_be_bytes(addr >> 2);
        let v = trie::get(&mem.data, state_hash, &key);
        if (Option::is_none(&v)) {
            if (addr < REG_FAKE_ADDRESS_START) {
                abort Errors::internal(addr)
            } else {
                0
            }
        } else {
            from_be_bytes(&Option::destroy_some(v))
        }
    }
    /// Write a u32 `value` as big-endian representation to memory addr start from `addr`
    public fun write_memory(mem: &mut Memory, state_hash: HashValue, addr: u64, value: u64): HashValue {
        assert!(addr & 3 == 0, Errors::invalid_argument(MEM_ACCESS_MUTST_BE_ALIGNED_TO_4BYTES));
        let value = to_be_bytes(value);
        let key = to_be_bytes(addr >> 2);
        let new_root = trie::update(&mut mem.data, state_hash, key, value);
        mem.root = new_root;
        new_root
    }

    public fun read_memory_bits(mem: &Memory, state_hash: HashValue, addr: u64): Bits {
        bits::from_u64(read_memory(mem, state_hash, addr), 32)
    }


    public fun write_memory_bits(mem: &mut Memory, state_hash: HashValue, addr: u64, bits: Bits): HashValue {
        assert!(bits::len(&bits) <= 32, Errors::invalid_argument(BITS_LEN_OUT_OF_BOUND));
        let full_32bits = bits::ze(bits, 32);
        write_memory(mem, state_hash, addr, bits::data(&full_32bits))
    }


    public fun read_reg(mem: &Memory, _state_hash: HashValue, reg_id: u64): u64 {
        get_register(mem, reg_id)
    }

    public fun read_reg_bits(mem: &Memory, state_hash: HashValue, reg_id: u64): Bits {
        bits::from_u64(read_reg(mem, state_hash, reg_id), 32)
    }


    public fun write_reg(mem: &mut Memory, state_hash: HashValue, reg_id: u64, v: u64): HashValue {
        set_register(mem, reg_id, v);
        state_hash
    }

    public fun write_reg_bits(mem: &mut Memory, state_hash: HashValue, reg_id: u64, v: Bits): HashValue {
        assert!(bits::len(&v) <= 32, Errors::invalid_argument(BITS_LEN_OUT_OF_BOUND));
        let full_32bits = bits::ze(v, 32);
        write_reg(mem, state_hash, reg_id, bits::data(&full_32bits))
    }


    /// to be bytes
    const U32_MAX: u64 = 0xffffffff;
    public fun to_be_bytes(v: u64): vector<u8> {
        assert!(v <= U32_MAX, 1001);
        // le bytes
        let ret = BCS::to_bytes(&v);
        // TODO: once u32 is added to move-lang, we can just delete the pop_backs
        Vector::pop_back(&mut ret);
        Vector::pop_back(&mut ret);
        Vector::pop_back(&mut ret);
        Vector::pop_back(&mut ret);
        Vector::reverse(&mut ret);
        ret
    }

    /// from be_bytes
    public fun from_be_bytes(dat: &vector<u8>): u64 {
        from_be_bytes_offset(dat, 0)
    }

    public fun from_be_bytes_offset(dat: &vector<u8>, offset: u64):u64 {
        let i = Vector::length(dat);
        let ret = 0;
        let pos = 0;
        while (i > offset && pos < 8) {
            i = i - 1;
            ret = ret | (*Vector::borrow(dat, i) as u64) << (pos * 8);
            pos = pos + 1;
        };
        ret
    }
}
