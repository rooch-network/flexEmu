//# init -n dev

//# faucet --addr alice --amount 100000000000

//# publish
module alice::TableWraper {
    use StarcoinFramework::Table;
    use trie::hash_value::HashValue;

    struct TableWrapper has key, store {
        data: Table::Table<HashValue, vector<u8>>
    }
    public fun new(): TableWrapper {
         TableWrapper {data: Table::new()}
    }
}

//# publish

module alice::TableTest{
    use StarcoinFramework::Option;
    use alice::TableWraper;
    use alice::TableWraper::TableWrapper;
    struct TableTest has key, store {
        data: Option::Option<TableWrapper>,
    }
    public fun create(signer: &signer) {
        move_to(signer, TableTest {data: Option::some( TableWraper::new())})
    }
}
//# run --signers alice
script {
    //use omo::mips_emulator;
    use StarcoinFramework::Signer;
    use alice::TableTest;
    fun main(signer: signer) {
        StarcoinFramework::Debug::print(&Signer::address_of(&signer));
        TableTest::create(&signer);
        // mips_emulator::create(&signer);
        // let emu_addr = Signer::address_of(&signer);
        // mips_emulator::batch_add_trie_data(emu_addr, access_nodes);
        // let new_root = mips_emulator::run(emu_addr, root_before);
        // assert!(new_root == root_after, 42);
    }
}
