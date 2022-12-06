//# init -n test

//# faucet --addr alice --amount 100000000000


//# read-json step-59/step-proof.json

//# run --signers alice --args x"{{$.read-json[-1].access_nodes}}" --args x"{{$.read-json[-1].root_before}}" --args x"{{$.read-json[-1].root_after}}"
script {
    use omo::mips_emulator;
    use StarcoinFramework::Signer;
    use trie::rlp;

    fun main(signer: signer, access_nodes: vector<u8>, root_before: vector<u8>, root_after: vector<u8>) {
        // StarcoinFramework::Debug::print(&Signer::address_of(&signer));
        //
        // StarcoinFramework::Debug::print(&access_nodes);
        // StarcoinFramework::Debug::print(&root_before);
        // StarcoinFramework::Debug::print(&root_after);

        let access_nodes = {
            let r = rlp::new(access_nodes);
            rlp::as_valuelist(&r)
        };

        mips_emulator::create(&signer);
        let emu_addr = Signer::address_of(&signer);

        mips_emulator::batch_add_trie_data(emu_addr, access_nodes);

        let new_root = mips_emulator::run(emu_addr, root_before);
        assert!(new_root == root_after, 42);
    }
}
