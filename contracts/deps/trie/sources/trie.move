module eth::trie {

    use extensions::table::Table;
    use std::hash::sha3_256 as keccak256;
    use trie::byte_utils;
    use extensions::table;
    use trie::hash_value::HashValue;
    use trie::hash_value;
    use trie::byte_utils::to_nibbles;
    use trie::rlp;
    use std::vector;
    use std::vector::length;
    use std::option::Option;
    use std::option;

    /// inline or hash.
    /// if hash, data.len must be 32.
    /// if inline, data.len must < 32.
    struct NodeId {
        inline: bool,
        data:vector<u8>,
    }
    struct TrieNode {
        encoded: vector<u8>,
        decoded: vector<vector<u8>>,
    }
    struct Trie {
        /// hash_of_value -> value
        data: Table<HashValue, vector<u8>>,
    }
    // Just a utility constant. RLP represents `NULL` as 0x80.
    const RLP_NULL_BYTES: u8 = 0x80;
    // TODO: fix this
    const KECCAK256_RLP_NULL_BYTES: vector<u8> = 0x80;

    fun update(trie: &mut Trie, key: vector<u8>, value: vector<u8>, root: HashValue): HashValue {
        // Special case when inserting the very first node.
        if (hash_value::bytes(&root) == &KECCAK256_RLP_NULL_BYTES) {
            return get_single_node_root_hash(trie, key,value)
        };
        let (proof, keyRemainder, path_value) = _walk_node_path(trie, root, &key );
        let newPath = _get_new_path(proof, length(&proof), key, keyRemainder, value);
        _get_updated_trie_root(newPath, key)
    }
    fun get(trie: &Trie, root: HashValue, key: &vector<u8> ): Option<vector<u8>> {
        let nibble_path = to_nibbles(key);
        let (walk_nodes, path_remainder, path_value) = _walk_node_path(trie,root, &nibble_path);
        let exists = (length(&path_remainder) == 0);

        // provided proof is not valid.
        assert!(
            (exists && option::is_some(&path_value)) || (!exists && option::is_none(&path_value)), 1000);
        path_value
    }

    fun _walk_node_path(trie: &Trie, root: HashValue, key: &vector<u8>): (vector<TrieNode>, vector<u8>, Option<vector<u8>>) {
        let nibble_path = to_nibbles(key);
        _walk_node_path_inner(trie, new_node_id(hash_value::to_bytes(root)), &nibble_path, 0, vector::empty<TrieNode>())
    }

    fun _walk_node_path_inner(trie: &Trie, node_id: NodeId, path_in_nibble: &vector<u8>, path_index: u64, proof: vector<TrieNode>): (vector<TrieNode>, vector<u8>, Option<vector<u8>>) {
        let current_node = get_trie_node(trie, node_id);

        vector::push_back(&mut proof, current_node);

        let node_elem_num = length(&current_node.decoded);

        if (node_elem_num == 17) {// branch node
            if (path_index == length(path_in_nibble)) {
                // We've hit the end of the key
                // meaning the value should be within this branch node.
                (proof, vector::empty<u8>(), option::some(*vector::borrow(&current_node.decoded, 16)))
            } else {
                // We're not at the end of the key yet.
                // Figure out what the next node ID should be and continue.
                let branch_key = *vector::borrow(path_in_nibble, path_index);
                let branch_elem = vector::borrow(&current_node.decoded, (branch_key as u64));
                let next_node_id = new_node_id( *branch_elem);
                _walk_node_path_inner(trie, next_node_id, path_in_nibble, path_index + 1, (move proof))
            }
        } else if (node_elem_num == 2) { // leaf or extension node
            let encoded_path = to_nibbles(vector::borrow(&current_node.decoded, 0));
            let prefix = *vector::borrow(&encoded_path, 0);
            let offset = 2 - prefix %2;
            let node_path = byte_utils::slice(&encoded_path, (offset as u64), length(&encoded_path));
            let path_remainder = byte_utils::slice(path_in_nibble, path_index, length(path_in_nibble));
            let shared_len = byte_utils::get_shared_length(&node_path, &path_remainder);
            if (prefix < 2) { // extension
                // Our extension shares some nibbles.
                // Carry on to the next node.
                if (shared_len == length(&node_path)) {
                    _walk_node_path_inner(trie, new_node_id(*vector::borrow(&current_node.decoded,1)), path_in_nibble, path_index + shared_len, (move proof))
                } else {
                    // Our extension node is not identical to the remainder.
                    // We've hit the end of this path
                    // updates will need to modify this extension.
                    (proof, byte_utils::slice(path_in_nibble, path_index, length(path_in_nibble)), option::none())
                }
            } else if (prefix < 4) { // leaf
                if (shared_len == length(&path_remainder) && shared_len == length(&node_path)) {
                    // The key within this leaf matches our key exactly.
                    // Increment the key index to reflect that we have no remainder.
                    (proof, vector::empty<>(), option::some(*vector::borrow(&current_node.decoded, 1)))
                } else {
                    // or else, insert should branch here, or get return none.
                    (proof, byte_utils::slice(path_in_nibble, path_index, length(path_in_nibble)), option::none())
                }
            } else {
                abort 10000
            };
        }
    }



    fun new_node_id(data: vector<u8>): NodeId {
        NodeId {
            inline: length(&data) < 32,
            data
        }
    }

    fun _get_new_path() {}
    fun _get_updated_trie_root() {}

    fun get_trie_node(trie: &Trie, node_id: NodeId): TrieNode {
        let node_data = if (node_id.inline) {
            assert!(vector::length(&node_id.data) < 32, 32);
            node_id.data

        } else {
            assert!(vector::length(&node_id.data) == 32, 32);
            let node_data = *table::borrow(&trie.data, hash_value::new(node_id.data));
            // bad hash in storage
            assert!(vector::length(&node_data) >= 32 &&  keccak256(node_data) == node_id.data, 1000);
            node_data
        };
        get_raw_node(node_data)
    }
    fun get_raw_node(encoded: vector<u8>): TrieNode {
        let decoded = rlp::decode_list(&encoded);
        //let decoded_len = vector::length(&decoded);
        TrieNode {
            encoded,
            decoded
        }

    }

     /// Computes the root hash for a trie with a single node.
     /// @param _key Key for the single node.
     /// @param _value Value for the single node.
     /// @return _updatedRoot Hash of the trie.
    fun get_single_node_root_hash(trie: &mut Trie, key: vector<u8>, value: vector<u8>): HashValue {
        let dat = make_leaf_node(byte_utils::to_nibbles(&key), value).encoded;
        let ret = keccak256(dat);
        table::add(&mut trie.data, hash_value::new(ret), dat);
        hash_value::new(ret)
    }

    fun make_leaf_node(key: vector<u8>, value: vector<u8>): TrieNode {

    }

    fun make_node(raw: vector<vector<u8>>): TrieNode {

    }
}
