module omo::mips_emulator {
    use omo::memory;
    use omo::mips;
    use trie::hash_value;

    public fun create(signer: &signer) {
        memory::create(signer);
    }
    public fun create_if_not_exists(signer: &signer) {
        memory::create_if_not_exists(signer);
    }
    public fun add_trie_data(emulator_addr: address, data: vector<u8>) {
        memory::batch_add_trie_data(emulator_addr, vector[data])
    }
    public fun batch_add_trie_data(emulator_addr: address, data: vector<vector<u8>>) {
        memory::batch_add_trie_data(emulator_addr, data)
    }

    /// Set register values, use u64 to represent (id: u32, value: u32)

    // public fun set_registers(emulator_addr: address, data: vector<u64>) {
    //     let mem = memory::get_mem(emulator_addr);
    //     {
    //         let i = 0;
    //         while (i < Vector::length(&data)) {
    //             let d = *Vector::borrow(&data, i);
    //             let id = d >> 32;
    //             let v = (d << 32) >> 32;
    //             memory::set_register(&mut mem, id, v);
    //             i = i + 1;
    //         }
    //     };
    //     memory::return_mem(mem)
    // }

    public fun run(emulator: address, state_root: vector<u8>): vector<u8> {
        let mem = memory::get_mem(emulator,  hash_value::new(state_root));
        let root = hash_value::new(state_root);
        mips::step(&mut mem, root);
        let new_root = memory::return_mem(mem);
        hash_value::to_bytes(new_root)
    }
}