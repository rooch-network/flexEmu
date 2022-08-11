module jellyfish_merkle_tree::jellyfish_merkle_tree {
    use extensions::table;
    use extensions::table::Table;
    use std::option;
    use std::option::Option;

    use jellyfish_merkle_tree::structure_hash;
    use jellyfish_merkle_tree::utils::to_nibbles;
    use jellyfish_merkle_tree::hash_value::HashValue;
    use jellyfish_merkle_tree::hash_value;
    use jellyfish_merkle_tree::node::{new_leaf, Blob, new_null, Node, new_blob};
    use jellyfish_merkle_tree::node;

    const Node_Null: u8 = 0;
    const Node_Internal: u8 = 1;
    const Node_Leaf: u8 = 2;
    const SPARSE_MERKLE_PLACEHOLDER_HASH: vector<u8> = b"fillme";
    struct Tree<K: store> has store, key {
        // hash_value -> node data
        values: Table<HashValue, Node<K>>,
    }

    // K only work for usual bcs-serializable data structure.
    public fun update<K: store+drop>(state_root: Option<HashValue>, structure: vector<u8>, key: K, value: vector<u8>) acquires Tree {
        let key_hash = structure_hash::hash(structure, &key);
        let storage = borrow_global_mut<Tree<K>>(@jellyfish_merkle_tree);
        let root = if (option::is_none(&state_root)) {
            let new_node =new_null<K>();
            let root = hash_value::new(SPARSE_MERKLE_PLACEHOLDER_HASH);
            table::add(&mut storage.values, copy root, new_node);
            root
        } else {
            option::destroy_some(state_root)
        };
        insert_at(storage, root, to_nibbles(&key_hash), key, new_blob(value));
    }

    fun insert_at<K: store+drop>(tree: &mut Tree<K>, root: HashValue, key_path: vector<u8>, key: K, value: Blob) {
        let root_node = table::borrow(&tree.values,copy root);
        let node_type = node::ty(root_node);
        if (node_type == Node_Null) {
            // delete old root
            let _ = table::remove(&mut tree.values, copy root);
            let leaf = new_leaf(key, value);

        } else if (node_type == Node_Internal) {

        } else if (node_type == Node_Leaf) {

        }
    }
    fun insert_new_leaf<K: store+drop>(tree: &mut Tree<K>, key: K, value: Blob) {
        let leaf = new_leaf(key, value);
        let leaf_hash = node::hash(&leaf);
        table::add(&mut tree.values, leaf_hash, leaf);
    }

}
module jellyfish_merkle_tree::node {
    use std::option::{Option,Self};
    use jellyfish_merkle_tree::hash_value::HashValue;
    use jellyfish_merkle_tree::hash_value;
    use jellyfish_merkle_tree::structure_hash;

    /// TODO: should we extract node data structure to its own module?
    struct Node<K> has store, drop {
        ty: u8,
        internal: Option<InternalNode>,
        leaf: Option<LeafNode<K>>,
    }
    struct LeafNode<K> has store, drop {
        key: K,
        blob_hash: HashValue,
        blob: Blob,
    }
    struct InternalNode has store, drop {
        children: vector<Child>,
    }
    struct Child has store, drop {
        hash: HashValue,
        is_leaf: bool
    }
    const Node_Null: u8 = 0;
    const Node_Internal: u8 = 1;
    const Node_Leaf: u8 = 2;


    // TODO: fill me

    const BLOB_STRUCTURE: vector<u8> = b"Blob";
    struct Blob has store, drop {
        blob: vector<u8>,
    }
    public fun new_blob(blob: vector<u8>): Blob {
        Blob {blob}
    }

    public fun new_null<K>(): Node<K> {
        Node {
            ty: Node_Null,
            internal: option::none(),
            leaf: option::none()
        }
    }

    public fun new_leaf<K>(key: K, blob: Blob): Node<K> {
        let blob_hash = structure_hash::hash(BLOB_STRUCTURE, &blob);
        Node {
            ty: Node_Leaf,
            leaf: option::some(LeafNode<K> {
                key,
                blob_hash: hash_value::new(blob_hash),
                blob
            }),
            internal: option::none(),
        }
    }

    public fun ty<K>(n: &Node<K>): u8 {
        n.ty
    }

    // TODO: impl me
    public fun hash<K>(n: &Node<K>): HashValue {
        abort 10
    }

}
module jellyfish_merkle_tree::hash_value {
    use std::vector;
    use jellyfish_merkle_tree::utils;

    struct HashValue has store, copy, drop {
        // 32 * 8
        value: vector<u8>,
    }

    public fun new(value: vector<u8>): HashValue {
        assert!(vector::length(&value) == 32, 1);
        HashValue {value}
    }
    public fun bytes(hash: &HashValue): &vector<u8> {
        &hash.value
    }
    // result to a 64-elems vector
    public fun to_nibbles(hash: &HashValue): vector<u8> {
        utils::to_nibbles(&hash.value)
    }

}

module jellyfish_merkle_tree::utils {
    use std::vector;
    public fun to_nibble(b: u8): (u8, u8) {
        let n1 = b >> 4;
        let n2 = (b << 4) >> 4;
        (n1, n2)
    }

    public fun to_nibbles(bytes: &vector<u8>): vector<u8> {
        let result = vector::empty<u8>();
        let i = 0;
        let data_len = vector::length(bytes);
        while (i < data_len) {
            let (a, b) = to_nibble(*vector::borrow(bytes, i));
            vector::push_back(&mut result, a);
            vector::push_back(&mut result, b);
            i = i + 1;
        };

        result
    }

}

module jellyfish_merkle_tree::structure_hash {
    use std::hash;
    use std::vector;
    use std::bcs;

    const STARCOIN_HASH_PREFIX: vector<u8> = b"STARCOIN::";

    public fun hash<MoveValue: store>(
        structure: vector<u8>,
        data: &MoveValue
    ): vector<u8> {
        let prefix_hash = hash::sha3_256(concat(&STARCOIN_HASH_PREFIX, structure));
        let bcs_bytes = bcs::to_bytes(data);
        hash::sha3_256(concat(&prefix_hash, bcs_bytes))
    }

    fun concat(v1: &vector<u8>, v2: vector<u8>): vector<u8> {
        let data = *v1;
        vector::append(&mut data, v2);
        data
    }
}
