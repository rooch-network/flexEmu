module omo::mips {

    use jellyfish_merkle_tree::hash_value::HashValue;
    use jellyfish_merkle_tree::hash_value;
    use std::vector;


    public fun read_memory(state_hash: HashValue, addr: u64): u64 {
        0
    }

    public fun write_memory(state_hash: HashValue, addr: u64, value: u64): HashValue {
        hash_value::new(vector::empty<u8>())
    }

    const REG_OFFSET: u64 = 0xc0000000;
    const REG_ZERO: u64 = 0;
    const REG_LR: u64 = 0x1f;
    const REG_PC: u64 = 0x20;
    const REG_HI: u64 = 0x21;
    const REG_LO: u64 = 0x22;
    const REG_HEAP: u64 = 0x23;

    const EXIT_ADDRESS: u64 = 0xffffffff;

    fun reg_to_mem_addr(reg_id: u64): u64 {
        REG_OFFSET + reg_id * 4
    }

    public fun step(state_hash: HashValue): HashValue {
        let pc = read_memory(state_hash, reg_to_mem_addr(REG_PC));
        if (pc == EXIT_ADDRESS) {
            return state_hash
        };
        let new_state = step_pc(state_hash, pc, pc + 4);
        new_state
    }

    fun step_pc(state_hash: HashValue, pc: u64, next_pc: u64): HashValue {
        let insn = read_memory(state_hash, pc);
        let opcode = insn >> 26; // first 6-bits
        //let func = insn & 0b111111; // last 6-bits
        // j-type j/jal
        if (opcode == 2 || opcode == 3) {
            //(next_pc & 0xf0000000) | (opcode);
            let new_state_hash = step_pc(state_hash, next_pc, next_pc + 4);
            if (opcode == 3) {
                state_hash = write_memory(state_hash, reg_to_mem_addr(REG_LR), pc + 8);
            };
            return state_hash;
        };
        return hash_value::new(vector::empty<u8>())
    }
}
module omo::utils {
    /// SignExt_idx(dat)
    public fun se(dat: u64, idx: u64): u64 {
        0
    }
}

/// Big Endian bits representation.
module omo::bits {

    /// 0x0000_0000_0000_0000
    struct Bits has copy, drop, store, key {
        data: u64,
        len: u8,
    }

    public fun from_u64(v: u64, len: u8): Bits {
        assert!(len <= 64, 1000);
        Bits {
            data: v,
            len
        }
    }

    #[test]
    fun test_repeat_bit() {
        assert!(repeat_bit(true, 1).data == 0x1, 0);
        assert!(repeat_bit(true, 10).data == 0x3ff, 1);
        assert!(repeat_bit(false, 1).data == 0x0, 2);
        assert!(repeat_bit(false, 10).data == 0x0, 3)
    }

    public fun repeat_bit(b: bool, n: u8): Bits {
        assert!(n <= 64, 1000);
        Bits {
            len: n,
            data: if (!b) {
                0
            } else {
                (1 << n) - 1
            }
        }
    }

    /// {X, Y}
    /// Concatenate the bits of X and Y together.
    /// Example: {10, 11, 011} = 1011011
    public fun concat(x: Bits, y: Bits): Bits {
        assert!(x.len + y.len <= 64, 1000);
        Bits {
            data: (x.data << y.len) | y.data,
            len: x.len + y.len
        }
    }


    #[test]
    fun test_concat() {
        let x = repeat_bit(true, 3);
        let y = repeat_bit(false, 2);
            {
                let r = concat(x, y);
                assert!(r.len == 5, 5);
                assert!(r.data == 0x1c, 12);
            };

            {
                let r = concat(y, x);
                assert!(r.len == 5, 5);
                assert!(r.data == 0x3, 3);
            };
    }


    /// X x Y
    /// Repeat bit X exactly Y times.
    /// Example: {1, 0 x 3} = 1000
    public fun repeat(x: Bits, n: u8): Bits {
        assert!(n != 0, 0);
        while (n > 1) {
            x = concat(x, x);
            n = n - 1
        };
        x
    }

    /// (X)[B:A]
    /// Slice bits A through B (inclusive) out of X.
    /// Example: (1100110101)[4:1] = 1010
    public fun slice(x: Bits, b: u8, a: u8): Bits {
        assert!(b >= a && b < 64, 1000);
        // 0b100000 - 1 = 0b11111
        let mask = (1 << (b + 1)) - 1;
        // (0b11111 >> 1) << 1 = 0b11110;
        let mask = ((mask >> a) << a);
        Bits {
            data: (x.data & mask) >> a,
            len: b - a + 1
        }
    }

    /// (X)[idx]
    /// Example:
    /// (0b1010101)[0] = 1
    /// (0b1010101)[1] = 0
    /// (0b1010101)[2] = 1
    public fun bit(x: &Bits, idx: u8): bool {
        assert!(idx < x.len, 1000);
        (x.data >> idx) % 2 == 1
    }

    public fun is_signed(x: &Bits): bool {
        bit(x, x.len - 1)
    }

    /// SignExt_Nb(X)
    /// Sign-extend X from N bits to 32 bits.
    /// SignExt_4b(1001) = {1 x 28, 1001}
    /// SignExt_4b(0111) = {0 x 28, 0111}
    public fun se(x: Bits, to: u8): Bits {
        assert!(to <= 64 && to >= x.len, 1000);
        let is_signed = is_signed(&x);
        concat(repeat_bit(is_signed, to - x.len), x)
    }
}
