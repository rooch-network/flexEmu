module trie::hash_value {
    use StarcoinFramework::Vector;
    use trie::byte_utils;

    struct HashValue has store, copy, drop {
        // 32 * 8
        value: vector<u8>,
    }

    public fun new(value: vector<u8>): HashValue {
        assert!(Vector::length(&value) == 32, 1);
        HashValue {value}
    }
    public fun bytes(hash: &HashValue): &vector<u8> {
        &hash.value
    }
    public fun to_bytes(hash: HashValue): vector<u8> {
        hash.value
    }
    // result to a 64-elems vector
    public fun to_nibbles(hash: &HashValue): vector<u8> {
        byte_utils::to_nibbles(&hash.value)
    }

}
