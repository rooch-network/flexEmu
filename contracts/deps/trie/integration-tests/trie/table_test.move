//# init -n dev

//# faucet --addr alice --amount 100000000000

//# publish
module alice::TableTest {
    use StarcoinFramework::Table;
    use trie::byte_utils;
    struct MyTable has key,store {
        t: Table::Table<u64, vector<u8>>,
    }

    public fun new(signer: &signer) {
        move_to(signer, MyTable {
            t: Table::new()
        });
        byte_utils::from_nibble(1,2);
    }
}

//# run --signers alice
script {
    use alice::TableTest;
    fun main(signer: signer) {
        TableTest::new(&signer);
    }
}