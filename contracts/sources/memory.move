module omo::memory {
    use trie::hash_value::HashValue;
    use signed_integer::bits::Bits;
    use StarcoinFramework::Vector::{length};
    use signed_integer::bits;
    use trie::trie;
    use StarcoinFramework::Vector;
    use StarcoinFramework::BCS;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Option;

    const REG_FAKE_ADDRESS_START: u64 = 0xffffffff + 1;
    struct MemoryStorage has key, store {
        data: Option::Option<trie::TrieDB>,
        registers: Option::Option<vector<Register>>,
    }
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
    }

    public fun create(signer: &signer) {
        move_to(signer, MemoryStorage {
            data: Option::some(trie::new()),
            registers: Option::some(Vector::empty())
        })
    }

    public fun get_mem(memory_addr: address): Memory acquires MemoryStorage {
         Memory {
             storage_handle: memory_addr,
             data: Option::extract(&mut borrow_global_mut<MemoryStorage>(memory_addr).data),
             registers: Option::extract(&mut borrow_global_mut<MemoryStorage>(memory_addr).registers),
         }
    }
    public fun borrow_db_mut(mem: &mut Memory): &mut trie::TrieDB {
        &mut mem.data
    }
    public fun set_register(mem: &mut Memory, id: u64, v: u64) {
        let i = 0;
        while (i < length(&mem.registers)) {
            let elem = Vector::borrow(&mem.registers, i);
            if (elem.id == id) {
                break
            };
            i = i+1;
        };
        if (i >= length(&mem.registers)) {
            Vector::push_back(&mut mem.registers, Register {id, value: v});
        } else {
            Vector::borrow_mut(&mut mem.registers, i).value = v;
        }
    }
    public fun get_register(mem: &Memory, id: u64): u64 {
        let i = 0;
        while (i < length(&mem.registers)) {
            let elem = Vector::borrow(&mem.registers, i);
            if (elem.id == id) {
                break
            };
            i = i+1;
        };
        if (i >= length(&mem.registers)) {

            0
        } else {
            Vector::borrow(&mem.registers, i).value
        }
    }

    public fun return_mem(mem: Memory)
    acquires MemoryStorage {
        let Memory {data, storage_handle, registers} = mem;
        Option::fill(&mut borrow_global_mut<MemoryStorage>(storage_handle).registers, registers);
        Option::fill(&mut borrow_global_mut<MemoryStorage>(storage_handle).data, data)
    }

    const MEM_ACCESS_MUTST_BE_ALIGNED_TO_4BYTES: u64 = 401;

    /// Read memory in four-bytes and convert it to u32 as big-endian representation.
    public fun read_memory(mem: &Memory, state_hash: HashValue, addr: u64): u64 {
        StarcoinFramework::Debug::print(&state_hash);
        StarcoinFramework::Debug::print(&addr);

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
        // StarcoinFramework::Debug::print(&state_hash);
        // if (addr > U32_MAX) {
        //     StarcoinFramework::Debug::print(&((addr - U32_MAX - 1) / 4));
        // } else {
        //     StarcoinFramework::Debug::print(&addr);
        // };
        // StarcoinFramework::Debug::print(&value);

        let value = to_be_bytes(value);
        let key = to_be_bytes(addr >> 2);
        trie::update(&mut mem.data, state_hash, key, value)
    }

    public fun read_memory_bits(mem: &Memory, state_hash: HashValue, addr: u64): Bits {
        bits::from_u64(read_memory(mem, state_hash, addr), 32)
    }

    const BITS_LEN_OUT_OF_BOUND: u64 = 601;
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
