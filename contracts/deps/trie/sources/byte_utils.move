module trie::byte_utils {
    use std::vector;
    use std::vector::length;


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
    public fun slice(
        data: &vector<u8>,
        start: u64,
        end: u64
    ): vector<u8> {
        let i = start;
        let result = vector::empty<u8>();
        let data_len = vector::length(data);
        let actual_end = if (end < data_len) {
            end
        } else {
            data_len
        };
        while (i < actual_end) {
            vector::push_back(&mut result, *vector::borrow(data, i));
            i = i + 1;
        };
        result
    }

    /// determines the number of elem shared between two bytes.
    public fun get_shared_length(a: &vector<u8>, b: &vector<u8>): u64 {
        let i = 0;
        let max_i = if( length(a) < length(b)) { length(a)} else {length(b)};
        while (i < max_i && vector::borrow(a, i) == vector::borrow(b, i)) {
            i = i+1;
        };
        i
    }
}
