module trie::rlp_decoder {
    use std::vector;
    use trie::rlp::unarrayify_integer;
    use trie::byte_utils::slice;


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
        let rets = vector::empty();
        let list_info = payload_info(&rlp.bytes, 0);
        let i = list_info.header_len;
        while (i < vector::length(&rlp.bytes)) {
            let info = payload_info(&rlp.bytes, i);
            let to_consume = info.header_len + info.value_len;
            vector::push_back(&mut rets, new(slice(&rlp.bytes, i, i + to_consume)));
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
        let first_byte = *vector::borrow(header_bytes, offset + 0);
        if (first_byte < 128) {
            PayloadInfo {header_len: 0, value_len: 1}
        } else if ( first_byte < 56 + 128) {
            PayloadInfo {header_len: 1, value_len: (first_byte as u64) - 128}
        } else if (first_byte < 192) {
            let len_of_len = first_byte - 183; // 183 = 128 + 56 - 1
            let length = unarrayify_integer(header_bytes, offset + 1, len_of_len);
            PayloadInfo {header_len: 1+ (len_of_len as u64), value_len: length}
        } else if (first_byte < 192 + 56) {
            PayloadInfo {header_len: 1, value_len: (first_byte as u64) - 192}
        } else {
            let len_of_len = first_byte - 247; // 247 = 192 + 56 - 1
            let length = unarrayify_integer(header_bytes, offset + 1, len_of_len);
            PayloadInfo {header_len: 1+ (len_of_len as u64), value_len: length}
        }
    }


    public fun is_null(rlp: &Rlp): bool {
        vector::is_empty(&rlp.bytes)
    }
    public fun is_empty(rlp: &Rlp): bool {
        !is_null(rlp) && ((*vector::borrow(&rlp.bytes, 0) == 192) || (*vector::borrow(&rlp.bytes, 0) == 128))
    }
    public fun is_list(rlp: &Rlp): bool {
        !is_null(rlp) && (*vector::borrow(&rlp.bytes, 0) >= 192)
    }
    public fun is_data(rlp: &Rlp): bool {
        !is_null(rlp) && (*vector::borrow(&rlp.bytes, 0) < 192)
    }

    fun decode_value(bytes: &vector<u8>): vector<u8> {
        let info  =payload_info(bytes, 0);
        slice(bytes, info.header_len, info.header_len + info.value_len)
    }
}