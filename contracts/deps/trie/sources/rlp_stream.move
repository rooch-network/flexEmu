module trie::rlp_stream {
    use StarcoinFramework::Vector;
    use StarcoinFramework::Option;
    use StarcoinFramework::BCS;

    use trie::byte_utils::{slice, slice_to_end};

    struct RlpStream has store, copy {
        buffer: vector<u8>,
        unfinished_list: vector<ListInfo>,
    }
    struct ListInfo has store, copy, drop {
        position: u64,
        current: u64,
        max: Option::Option<u64>,
    }
    public fun new(): RlpStream {
        RlpStream {
            buffer: Vector::empty(),
            unfinished_list: Vector::empty(),
        }
    }
    public fun new_list(len: u64): RlpStream {
        let encoder = new();
        begin_list(&mut encoder, len);
        encoder
    }
    public fun is_finished(encoder: &RlpStream): bool {
        Vector::is_empty(&encoder.unfinished_list)
    }

    /// Streams out encoded bytes.
    /// abort is stream is not finished
    public fun out(encoder: RlpStream): vector<u8> {
        assert!(is_finished(&encoder), 1000);
        let RlpStream {buffer, unfinished_list: _} = encoder;
        buffer
    }

    public fun append_empty_data(encoder: &mut RlpStream) {
        append(encoder, Vector::empty());
    }
    /// Appends value to the end of stream.
    public fun append(encoder: &mut RlpStream, value: vector<u8>) {
        let encoded_value = encode_value(&value);
        Vector::append(&mut encoder.buffer, encoded_value);
        note_appended(encoder, 1);
    }

    /// Appends raw (pre-serialised) RLP data. Use with caution.
    public fun append_raw(encoder: &mut RlpStream, bytes: vector<u8>, item_count: u64) {
        Vector::append(&mut encoder.buffer, bytes);
        note_appended(encoder, item_count);
    }

    fun begin_list(encoder: &mut RlpStream, len: u64) {
        if (len == 0) {
            Vector::push_back(&mut encoder.buffer, 192u8);
        } else {
            // payload is longer than 1 byte only for lists > 55 bytes
            // by pushing always this 1 byte we may avoid unnecessary shift of data
            Vector::push_back(&mut encoder.buffer, 0);
            let pos = Vector::length(&encoder.buffer);
            let new_unfinished_list = ListInfo {
                position: pos,
                current: 0,
                max: Option::some(len)
            };
            Vector::push_back(&mut encoder.unfinished_list, new_unfinished_list);
        }
    }

    fun note_appended(encoder: &mut RlpStream, inserted_items: u64) {
       if (Vector::is_empty(&encoder.unfinished_list)) {
           return
       };
        assert!(!Vector::is_empty(&encoder.unfinished_list), 1000);
        let current_list = Vector::pop_back(&mut encoder.unfinished_list);

        current_list.current = current_list.current + inserted_items;
        let should_finish = if (Option::is_some(&current_list.max)) {
            let max_items = *Option::borrow(&current_list.max);
            if (current_list.current > max_items) {
                abort 1000
            };
            current_list.current == max_items
        } else {false};

        if (should_finish) {
            let len = Vector::length(&encoder.buffer) - current_list.position;
            if (len < 56) {
                *Vector::borrow_mut(&mut encoder.buffer, current_list.position - 1) = (len as u8) + 192;
            } else {
                let output = slice(&encoder.buffer, 0, current_list.position);
                let length_BE = encode_size(len);
                let length_BE_len = Vector::length(&length_BE);
                *Vector::borrow_mut(&mut output, current_list.position - 1) = (length_BE_len as u8) + 247u8;
                Vector::append<u8>(&mut output, length_BE);
                Vector::append<u8>(&mut output, slice_to_end(&encoder.buffer, current_list.position));
                encoder.buffer = output;
            }
        } else {
            Vector::push_back(&mut encoder.unfinished_list, current_list);
        }

    }
    public fun encode_value(data: &vector<u8>): vector<u8> {
        let data_len = Vector::length(data);
        let rlp = Vector::empty<u8>();
        if (data_len == 1 && *Vector::borrow(data, 0) < 128u8) {
            Vector::append<u8>(&mut rlp, *data);
        } else if (data_len < 56) {
            Vector::push_back<u8>(&mut rlp, (data_len as u8) + 128u8);
            Vector::append<u8>(&mut rlp, *data);
        } else {
            let length_BE = encode_size(data_len);
            let length_BE_len = Vector::length(&length_BE);
            Vector::push_back<u8>(&mut rlp, (length_BE_len as u8) + 183u8);
            Vector::append<u8>(&mut rlp, length_BE);
            Vector::append<u8>(&mut rlp, *data);
        };
        rlp
    }
    fun encode_size(len: u64): vector<u8> {
        // BCS int is in little endian.
        let bytes: vector<u8> = BCS::to_bytes(&len);

        // remove trailing zero bytes.
        let i = Vector::length(&bytes);
        while (i > 0) {
            let last_elem = Vector::pop_back(&mut bytes);
            if (last_elem != 0) {
                Vector::push_back(&mut bytes, last_elem);
                break
            };
            i = i - 1;
        };
        Vector::reverse(&mut bytes);
        bytes
    }


    #[test]
    fun test_encoding() {
        {
            let encoder = new();
            append_empty_data(&mut encoder);
            let encoded_data = out(encoder);
            assert!(encoded_data == vector[0x80], 0x80);
        };
        {
            let data = x"f84d0589010efbef67941f79b2a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470";
            let encoder = new_list(4);
            append(&mut encoder, x"05");
            append(&mut encoder,x"010efbef67941f79b2");
            append(&mut encoder,x"56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421");
            append(&mut encoder,x"c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
            let encoded = out(encoder);
            assert!(data == encoded, 1000);
        };
    }
}