//# init -n dev

//# faucet --addr alice --amount 100000000000


//# run --signers alice
script {
    use trie::trie;
    use trie::hash_value;
    use StarcoinFramework::Hash;
    use StarcoinFramework::Option;

    fun main(signer: signer) {
        let db = trie::new();

        let root = hash_value::new(Hash::keccak_256(vector[0x80]));

        {
            root = trie::update(&mut db, root, b"foo", b"foo");
            assert!(hash_value::to_bytes(root) == x"51d8ccee4184b078b508033281a3dc892194afc17b3e92ae7e4a5b400e8454cc", 1);
            let v = trie::get(&db, root, &b"foo");
            assert!(Option::is_some(&v), 11);
            assert!(Option::destroy_some(v) == b"foo", 12);
        };

        {
            root = trie::update(&mut db, root, b"fooo", b"fooo");
            assert!(hash_value::to_bytes(root) == x"a6a751b890341768940a99f4e6b337a3c279e014fc0980a4d96ec72225567add", 2);
            let v = trie::get(&db, root, &b"fooo");
            assert!(Option::is_some(&v), 21);
            assert!(Option::destroy_some(v) == b"fooo", 22);
        };

        {
            root = trie::update(&mut db, root, b"foa", b"foa");
            assert!(hash_value::to_bytes(root) == x"227cd158eb4ad8a5169fdbd13c7d906ccf28937d21cea2fa7635e941c7c5cc65", 3);
        };

        {
            root = trie::update(&mut db, root, b"fooa", b"fooa");
            assert!(hash_value::to_bytes(root) == x"87b08ece907edf5c5c19e56beb0ed9badf7bbec61f5e686ca0e31a220e0d4b19", 4);
        };
        {
            root = trie::update(&mut db, root, b"fooa", b"foob");
            assert!(hash_value::to_bytes(root) == x"55f7f9d2d7117ebefcfc94b0c3b526508ecff533e8f1b0405ff22f6f5c73ebd2", 5);
        };
        trie::save(&signer, db);
    }
}