module signed_integer::i64 {

    use signed_integer::bits::Bits;
    use signed_integer::bits;

    struct I64 has copy, drop, store, key {
        bits: u64,
    }

    /// 2**63
    const MIN_V: u64 = 0x8000000000000000;
    /// 2**63 - 1
    const MAX_V: u64 = 0x8000000000000000 - 1;

    const MASK: u64 = 0xffffffffffffffff;
    const BIT_LEN: u8 = 64;

    public fun new(v: u64, positive: bool): I64 {
        if (v==0 || positive) {
            assert!(v <= MAX_V, 1000);
            I64 {
                bits: v,
            }
        } else {
            assert!(v <= MIN_V, 1000);
            // 2'complement of v
            I64 {
                bits: two_complement(v)
            }
        }
    }
    public fun from_u64(v: u64): I64 {
        new(v, true)
    }

    public fun from_bits(bits: Bits): I64 {
        I64 {
            bits: bits::data(&bits::se(bits, 64))
        }
    }
    public fun to_bits(v: I64): Bits {
        bits::from_u64(v.bits, 64)
    }

    public fun zero(): I64 {
        I64 {
            bits: 0,
        }
    }

    public fun max(): I64 {
        new(MAX_V, true)
    }

    public fun min(): I64 {
        new(MIN_V, false)
    }

    public fun bits(v: I64): u64 {
        v.bits
    }

    // return `-v`
    public fun negative(v: I64): I64 {
        if (v.bits == 0) {
            return v
        };

        if (v.bits == (1<<63)) {
            abort 1000
        };
        I64 {
            bits: two_complement(v.bits)
        }

    }


    //// Equal methods
    public fun less_than(a: I64, b: I64): bool {
        let a_msb = (a.bits >> (BIT_LEN - 1));
        let b_msb = (b.bits >> (BIT_LEN - 1));
        if (a_msb == b_msb) {
            a.bits < b.bits
        } else if (a_msb == 0) {
            false
        } else {
            true
        }
    }


    public fun abs(v: I64): u64 {
        if (positive(v)) {
            v.bits
        } else {
            two_complement(v.bits)
        }
    }

    public fun positive(v: I64): bool {
        (v.bits >> (BIT_LEN - 1)) == 0
    }

    public fun eq(a: I64, b: I64): bool {
        a.bits == b.bits
    }


    public fun add(a: I64, b: I64): I64 {
        let (overflow, v) = add_2c(a.bits, b.bits);
        if (overflow) {
            abort  10000
        };
        I64 {bits: v}

    }

    fun two_complement(v: u64): u64 {
        (v ^ MASK) + 1
    }

    // 2-complement add, return (overflow, value)
    fun add_2c(a: u64, b: u64): (bool, u64) {
        let a = (a as u128);
        let b = (b as u128);
        let v = a + (a >> 63 << 64) + b + (b >> 63 << 64);
        let lo = ((v & 0xffffffffffffffff) as u64);
        let hi = ((v >> 64) as u64);
        ((lo >> 63) != (hi & 0x1), lo)
    }

    public fun sub(a: I64, b: I64): I64 {
        let (overflow, v) = add_2c(a.bits, two_complement(b.bits));
        if (overflow) {
            abort 10000
        };
        I64 {bits: v}
    }


    public fun mul(a: I64, b: I64): I64 {
        let v = abs(a) * abs(b);
        new(v, positive(a) == positive(b))
    }

    // FIXME: wrong
    public fun div(a: I64, b: I64): I64 {
        let v = abs(a) / abs(b);
        new(v, positive(a) == positive(b))
    }
    public fun rem(a: I64, b: I64): I64 {
        let v = abs(a) % abs(b);
        new(v, positive(a) == positive(b))
    }

    #[test]
    fun test_arith() {
        let a = new(2, true);
        let b= new(3, true);

            {
                assert!(add(a, b) == from_u64(5), 11);
                assert!(add(a, negative(b)) == new(1, false), 12);
                assert!(add(b, negative(a)) == new(1, true), 13);
                assert!(add(negative(a), negative(b)) == new(5, false), 13);
            };
            {
                assert!(sub(a, b) == new(1, false), 11);
                assert!(sub(a, negative(b)) == new(5, true), 12);
                assert!(sub(negative(b), a) == new(5, false), 13);
                assert!(sub(negative(a), negative(b)) == new(1, true), 13);
            };

            {
                assert!(mul(a, b) == from_u64(6), 1);
                assert!(mul(negative(a), b) == new(6, false), 2);
                assert!(mul(a, negative(b)) == new(6, false), 3);
                assert!(mul(negative(a), negative(b)) == new(6, true), 4);
            };
            {
                assert!(div(a, b) == new(0, true), 1);
                assert!(div(b, a) == new(1, true), 2);
                assert!(div(negative(a), b) == new(0, true), 3);
                assert!(div(negative(b), a) == new(1, false), 4);
                assert!(div(negative(b), negative(a)) == new(1, true), 5);
            };
    }

    #[test]
    fun test_less_than() {
        assert!(less_than(from_u64(1), from_u64(2)), 100);
        assert!(!less_than(from_u64(2), from_u64(1)), 100);
        assert!(!less_than(new(2, true), new(1, false)), 100);
    }

    #[test]
    fun test_negative() {
        assert!(negative(from_u64(1)) == new(1, false), 100);
        //assert!(from_u64(1) == negative(new(1, false)), 200);
    }

    #[test]
    #[expected_failure]
    fun test_negative_err() {
        assert!(negative(from_u64(1)) == from_u64(1), 100);
    }


    #[test]
    fun test_add() {
        assert!(add(new(1, true), new(MAX_V - 1, true)) == max(), 100);
        assert!(add(new(1, false), new(1, true)) == zero(), 100);
        assert!(add(new(1, false), new(MIN_V - 1, false)) == min(), 100);
        assert!(add(new(1, true), new(2, false)) == new(1, false), 100);
    }

    #[test]
    #[expected_failure]
    fun test_add_err() {
        add(new(1, true), new(MAX_V, true));
    }

    #[test]
    #[expected_failure]
    fun test_add_err1() {
        add(new(1, false), new(MIN_V, false));
    }


    #[test]
    fun test_check_v_ok() {
        new(MAX_V, true);
        new(MIN_V, false);
        assert!(new(0, true) == new(0, false), 1000);
        assert!(negative(new(MAX_V, true)) == new(MIN_V - 1, false), 1001);
    }

    #[test]
    #[expected_failure]
    fun test_negative_of_min_v_err() {
        negative(new(MIN_V, false));
    }

    #[test]
    fun test_add_and_bits() {
        // 0b1111_1111_0101_0110
        let a = new(0xaa, false);
        // 0b0000_0000_1010_1010
        let b = new(0xaa, true);
        assert!(add(a, b) == zero(), 0);
        assert!(((a.bits as u128) + (b.bits as u128)) & 0xffffffff == 0, 1);
    }
}
