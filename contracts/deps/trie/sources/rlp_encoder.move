module trie::rlp_encoder {
    use std::vector;
    use std::option;
    use trie::rlp;
    use trie::rlp::encode_integer_in_big_endian;

    struct Encoder has store, copy {
        buffer: vector<u8>,
        unfinished_list: vector<ListInfo>,
    }
    struct ListInfo has store, copy, drop {
        position: u64,
        current: u64,
        max: option::Option<u64>,
    }
    public fun new(): Encoder {
        Encoder {
            buffer: vector::empty(),
            unfinished_list: vector::empty(),
        }
    }
    public fun new_list(len: u64): Encoder {
        let encoder = new();
        begin_list(&mut encoder, len);
        encoder
    }
    public fun is_finished(encoder: &Encoder): bool {
        vector::is_empty(&encoder.unfinished_list)
    }

    /// Streams out encoded bytes.
    /// abort is stream is not finished
    public fun out(encoder: Encoder): vector<u8> {
        assert!(is_finished(&encoder), 1000);
        let Encoder {buffer, unfinished_list: _} = encoder;
        buffer
    }

    public fun append_empty_data(encoder: &mut Encoder) {
        append(encoder, vector::empty());
    }
    /// Appends value to the end of stream.
    public fun append(encoder: &mut Encoder, value: vector<u8>) {
        let encoded_value = rlp::encode(&value);
        vector::append(&mut encoder.buffer, encoded_value);
        note_appended(encoder, 1);
    }

    /// Appends raw (pre-serialised) RLP data. Use with caution.
    public fun append_raw(encoder: &mut Encoder, bytes: vector<u8>, item_count: u64) {
        vector::append(&mut encoder.buffer, bytes);
        note_appended(encoder, item_count);
    }

    fun begin_list(encoder: &mut Encoder, len: u64) {
        if (len == 0) {
            vector::push_back(&mut encoder.buffer, 192u8);
        } else {
            // payload is longer than 1 byte only for lists > 55 bytes
            // by pushing always this 1 byte we may avoid unnecessary shift of data
            vector::push_back(&mut encoder.buffer, 0);
            let pos = vector::length(&encoder.buffer);
            let new_unfinished_list = ListInfo {
                position: pos,
                current: 0,
                max: option::some(len)
            };
            vector::push_back(&mut encoder.unfinished_list, new_unfinished_list);
        }
    }

    fun note_appended(encoder: &mut Encoder, inserted_items: u64) {
//        if (vector::is_empty(&encoder.unfinished_list)) {
//            return
//        };
        assert!(!vector::is_empty(&encoder.unfinished_list), 1000);
        let current_list = vector::pop_back(&mut encoder.unfinished_list);

        current_list.current = current_list.current + inserted_items;
        let should_finish = if (option::is_some(&current_list.max)) {
            let max_items = *option::borrow(&current_list.max);
            if (current_list.current > max_items) {
                abort 1000
            };
            current_list.current == max_items
        } else {false};

        if (should_finish) {
            let len = vector::length(&encoder.buffer) - current_list.position;
            if (len < 56) {
                *vector::borrow_mut(&mut encoder.buffer, current_list.position - 1) = (len as u8) + 192;
            } else {
                let output = vector::empty<u8>();
                let length_BE = encode_integer_in_big_endian(len);
                let length_BE_len = vector::length(&length_BE);
                vector::push_back<u8>(&mut output, (length_BE_len as u8) + 247u8);
                vector::append<u8>(&mut output, length_BE);
                vector::append<u8>(&mut output, encoder.buffer);
                encoder.buffer = output;
            }
        } else {
            vector::push_back(&mut encoder.unfinished_list, current_list);
        }

    }
    public fun encode_value(value: &vector<u8>): vector<u8> {
        rlp::encode(value)
    }
}