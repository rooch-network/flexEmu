/// Big Endian bits representation.
module Std::bits {

    const MAX_LEN: u8 = 64;
    /// 0x0000_0000_0000_0000
    struct Bits has copy, drop, store, key {
        data: u64,
        len: u8,
    }

    public fun zero(): Bits {
        Bits {
            data: 0,
            len: 0
        }
    }

    public fun from_u64(v: u64, len: u8): Bits {
        assert!(len <= 64, 1000);
        Bits {
            data: v,
            len
        }
    }

    public fun len(v: &Bits): u8 {
        v.len
    }
    public fun data(v: &Bits): u64 {
        v.data
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
                ((1u128 << n - 1) as u64)
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
                assert!(r.data == 7, 7);
            };
            {
                let zero = zero();
                assert!(concat(zero, x).data == 7, 7);
                assert!(concat(zero, x).len == x.len, 3);
                assert!(concat(x, zero).data == 7, 7);
                assert!(concat(x, zero).len == x.len, 3);
            }
    }


    /// X x Y
    /// Repeat bit X exactly Y times.
    /// Example: {1, 0 x 3} = 1000
    public fun repeat(x: Bits, n: u8): Bits {
        assert!(n != 0, 0);
        let half = n / 2;
        let remain = n % 2;
        let a = if (half > 0) {
            let half = repeat(x, half);
            concat(half, half)
        } else {
            zero()
        };
        let b = if (remain == 0) {
            zero()
        } else {
            x
        };
        concat(a, b)
    }

    #[test]
    fun test_repeat() {
        // 0b0001_0010
        let e = from_u64(0x12, 8);
        let ee = repeat(e, 8);
        assert!(ee.len == 64, 64);
        assert!(ee.data == 0x1212121212121212, ee.data);
    }


    /// (X)[B:A]
    /// Slice bits A through B (inclusive) out of X.
    /// Example: (1100110101)[4:1] = 1010
    public fun slice(x: Bits, b: u8, a: u8): Bits {
        assert!(b >= a && b < x.len, 1000);
        // 0b100000 - 1 = 0b11111
        let mask = (1 << (b + 1)) - 1;
        // (0b11111 >> 1) << 1 = 0b11110;
        let mask = ((mask >> a) << a);
        Bits {
            data: (x.data & mask) >> a,
            len: b - a + 1
        }
    }

    #[test]
    fun test_slice() {
        // 0b0011_0101
        let x = from_u64(0x35, 8);

            {
                let s = slice(x, 4, 1);
                assert!(s.len == 4, 4);
                assert!(s.data == 10, 10);
            };
            {
                let s = slice(x, 7, 1);
                assert!(s.len == 7, 7);
                // 0b0011_010
                assert!(s.data == 0x1a, 0x1a);
            };

    }
    /// (X)[B:A] = Y
    /// Write `y` to x[b:a]
    public fun write_range(x: Bits, b: u8, a: u8, y: Bits): Bits {
        assert!(b >= a && b < x.len, 1000);
        assert!(1 + b - a == y.len, 1001);

        let mask = (1 << (b+1)) - 1;
        let mask = (mask >> a) << a;
        Bits {
            data: (x.data & (mask ^ 0xffffffffffffffff)) | (y.data << a),
            len: x.len
        }
    }
    #[test]
    fun test_write_range() {
        // 0b0011_0101
        let x = from_u64(0x35, 8);

            {
                let s = write_range(x, 3, 0, from_u64(0x0, 4));
                assert!(s.len == 8, 8);
                assert!(s.data == 0x30, s.data);
            };
            {
                let s = write_range(x, 7, 4, from_u64(0xf, 4));
                assert!(s.len == 8, 8);
                // 0b0011_010
                assert!(s.data == 0xf5, 0xf5);
            };
            {
                let s = write_range(x, 7, 0, from_u64(0x1f, 8));
                assert!(s.len == 8, 8);
                // 0b0011_010
                assert!(s.data == 0x1f, 0x1f);
            };
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

    #[test]
    fun test_get_bit() {
        // 0b1010_1010
        let x = from_u64(0xaa, 8);
        let i = 0;
        while (i < 8) {
            assert!(bit(&x, i) == (i%2!=0), ((i % 2) as u64));
            i = i+1;
        };
    }

    public fun is_signed(x: &Bits): bool {
        bit(x, x.len - 1)
    }

    #[test]
    fun test_is_signed() {
        let x = repeat_bit(true, 1);
        assert!(is_signed(&x), 1);
        assert!(!is_signed(&repeat_bit(false, 1)), 0);
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

    /// Zero ext `x` to `to` len
    public fun ze(x: Bits, to: u8): Bits {
        concat(repeat_bit(false, to - x.len), x)
    }

    #[test]
    fun test_se() {
        // 0b_1001
        let x = from_u64(9, 4);
        let se = se(x, 32);
        assert!(se.len == 32, (se.len as u64));
        assert!(se.data == 0xfffffff9, se.data);
    }

    public fun right_shift(x: Bits, n: u8): Bits {
        assert!(n <= x.len, 1000);
        Bits {
            data: x.data >> n,
            len: x.len - n
        }
    }
    public fun left_shift(x: Bits, n: u8): Bits {
        assert!(n <= MAX_LEN, 1000);
        Bits {
            data: x.data << n,
            len: if (x.len + n < MAX_LEN) {x.len +n} else {MAX_LEN},
        }
    }
    // TODO: add test about shift
}
