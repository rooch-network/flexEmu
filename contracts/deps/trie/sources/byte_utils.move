module trie::byte_utils {
    use StarcoinFramework::Vector;
    use StarcoinFramework::Vector::length;


    public fun to_nibble(b: u8): (u8, u8) {
        let n1 = b >> 4;
        let n2 = (b << 4) >> 4;
        (n1, n2)
    }

    public fun to_nibbles(bytes: &vector<u8>): vector<u8> {
        let result = Vector::empty<u8>();
        let i = 0;
        let data_len = Vector::length(bytes);
        while (i < data_len) {
            let (a, b) = to_nibble(*Vector::borrow(bytes, i));
            Vector::push_back(&mut result, a);
            Vector::push_back(&mut result, b);
            i = i + 1;
        };

        result
    }

    public fun from_nibble(a: u8, b: u8): u8 {
        (a << 4) + b
    }

    public fun from_nibbles(nibbles: &vector<u8>): vector<u8> {
        assert!(length(nibbles) %2 == 0, 1000);
        let ret_len = length(nibbles) / 2;
        let ret_v = Vector::empty<u8>();
        let i = 0;
        while (i < ret_len) {
            let a = *Vector::borrow(nibbles, i * 2);
            let b = *Vector::borrow(nibbles, i*2+1);
            Vector::push_back(&mut ret_v, (a << 4) + b);
            i = i + 1;
        };
        ret_v
    }

    public fun slice_to_end(data: &vector<u8>, from: u64): vector<u8> {
        // short cut for slice whole range
        if (from == 0) {
            return *data
        };
        slice(data, from, length(data))
    }

    public fun slice(
        data: &vector<u8>,
        start: u64,
        end: u64
    ): vector<u8> {
        let i = start;
        let result = Vector::empty<u8>();
        let data_len = Vector::length(data);
        let actual_end = if (end < data_len) {
            end
        } else {
            data_len
        };
        while (i < actual_end) {
            Vector::push_back(&mut result, *Vector::borrow(data, i));
            i = i + 1;
        };
        result
    }

    /// determines the number of elem shared between two bytes.
    public fun get_shared_length(a: &vector<u8>, b: &vector<u8>): u64 {
        let i = 0;
        let max_i = if( length(a) < length(b)) { length(a)} else {length(b)};
        while (i < max_i && Vector::borrow(a, i) == Vector::borrow(b, i)) {
            i = i+1;
        };
        i
    }
}
