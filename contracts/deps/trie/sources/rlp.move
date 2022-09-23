module trie::rlp {
    use StarcoinFramework::Vector;
    use trie::byte_utils::slice;
    use StarcoinFramework::Vector::length;


    struct Rlp has copy, drop {
        bytes: vector<u8>,
    }
    struct PayloadInfo has copy, drop {
        header_len: u64,
        value_len: u64
    }

    public fun new(data: vector<u8>): Rlp {
        Rlp {bytes: data}
    }
    public fun raw(rlp: &Rlp): vector<u8> {
        rlp.bytes
    }
    const RlpExpectedToBeList: u64 = 1000;
    const RlpExpectedToBeData: u64 = 1001;
    public fun as_val(rlp: &Rlp): vector<u8> {
        assert!(is_data(rlp), RlpExpectedToBeData);
        decode_value(&rlp.bytes)
    }
    public fun as_list(rlp: &Rlp): vector<Rlp> {
        assert!(is_list(rlp), RlpExpectedToBeList);
        let rets = Vector::empty();
        let list_info = payload_info(&rlp.bytes, 0);
        let i = list_info.header_len;
        while (i < length(&rlp.bytes)) {
            let info = payload_info(&rlp.bytes, i);
            let to_consume = info.header_len + info.value_len;
            Vector::push_back(&mut rets, new(slice(&rlp.bytes, i, i + to_consume)));
            i = i + to_consume;
        };
        rets
    }


    /// Returns an Rlp item in a list at the given index.
    public fun at(rlp: &Rlp, index: u64): Rlp {
        assert!(is_list(rlp), RlpExpectedToBeList);
        let offset = 0;
        let list_info = payload_info(&rlp.bytes, offset);
        offset = offset + list_info.header_len;

        let consumed = consume_items(&rlp.bytes, offset, index);

        offset = offset + consumed;
        let found = payload_info(&rlp.bytes, offset);
        let data  = slice( &rlp.bytes, offset, offset + found.header_len + found.value_len);
        new(data)
    }

    fun consume_items(bytes: &vector<u8>,offset: u64, items: u64): u64 {
        let i = 0;
        let consumed = 0;
        while (i < items) {
            let info = payload_info(bytes, offset);
            let to_consume = (info.header_len + info.value_len);
            offset = offset + to_consume;
            consumed = consumed + to_consume;
            i = i+1;
        };
        consumed
    }

    /// use (vector, offset) to emulate slice
    public fun payload_info(header_bytes: &vector<u8>,offset: u64): PayloadInfo {
        let first_byte = *Vector::borrow(header_bytes, offset + 0);
        if (first_byte < 128) {
            PayloadInfo {header_len: 0, value_len: 1}
        } else if ( first_byte < 56 + 128) {
            PayloadInfo {header_len: 1, value_len: (first_byte as u64) - 128}
        } else if (first_byte < 192) {
            let len_of_len = first_byte - 183; // 183 = 128 + 56 - 1
            let length = decode_size(header_bytes, offset + 1, len_of_len);
            PayloadInfo {header_len: 1+ (len_of_len as u64), value_len: length}
        } else if (first_byte < 192 + 56) {
            PayloadInfo {header_len: 1, value_len: (first_byte as u64) - 192}
        } else {
            let len_of_len = first_byte - 247; // 247 = 192 + 56 - 1
            let length = decode_size(header_bytes, offset + 1, len_of_len);
            PayloadInfo {header_len: 1+ (len_of_len as u64), value_len: length}
        }
    }
    public fun decode_size(
        data: &vector<u8>,
        offset: u64,
        size_len: u8
    ): u64 {
        let result = 0;
        let i = 0u8;
        while (i < size_len) {
            result = result * 256 + (*Vector::borrow(data, offset + (i as u64)) as u64);
            i = i + 1;
        };
        result
    }



    public fun is_null(rlp: &Rlp): bool {
        Vector::is_empty(&rlp.bytes)
    }
    public fun is_empty(rlp: &Rlp): bool {
        !is_null(rlp) && ((*Vector::borrow(&rlp.bytes, 0) == 192) || (*Vector::borrow(&rlp.bytes, 0) == 128))
    }
    public fun is_list(rlp: &Rlp): bool {
        !is_null(rlp) && (*Vector::borrow(&rlp.bytes, 0) >= 192)
    }
    public fun is_data(rlp: &Rlp): bool {
        !is_null(rlp) && (*Vector::borrow(&rlp.bytes, 0) < 192)
    }

    fun decode_value(bytes: &vector<u8>): vector<u8> {
        let info  =payload_info(bytes, 0);
        slice(bytes, info.header_len, info.header_len + info.value_len)
    }

    #[test]
    fun test_decoding() {
        {
            let data = x"f84d0589010efbef67941f79b2a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
            let rlp = new(data);
            let elems = as_list(&rlp);
            assert!(length(&elems) == 4, 4);
            assert!(as_val(&Vector::pop_back(&mut elems)) ==  x"c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470", 1001);
            assert!(as_val(&Vector::pop_back(&mut elems)) == x"56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421", 1002);
            assert!(as_val(&Vector::pop_back(&mut elems)) == x"010efbef67941f79b2", 1003);
            assert!(as_val(&Vector::pop_back(&mut elems)) == x"05", 1004);
        };
    }
}