module Std::i128 {
    struct I128 has copy, drop, store, key {
        positive: bool,
        v: u128,
    }

    /// 2**127
    const MIN_V: u128 = 0x80000000000000000000000000000000;
    /// 2**127 - 1
    const MAX_V: u128 = 0x80000000000000000000000000000000 - 1;

    public fun new(v: u128, positive: bool): I128 {
        let n = I128 {
            positive,
            v
        };
        check_v(n)
    }

    public fun from_u128(v: u128): I128 {
        check_v(I128 { positive: true, v })
    }

    public fun zero(): I128 {
        I128 {
            positive: true,
            v: 0,
        }
    }

    public fun max(): I128 {
        new(MAX_V, true)
    }

    public fun min(): I128 {
        new(MIN_V, false)
    }

    // return `-v`
    public fun negative(v: I128): I128 {
        v.positive = !v.positive;
        check_v(v)
    }


    //// Equal methods
    public fun less_than(a: I128, b: I128): bool {
        if (a.positive) {
            if (b.positive) {
                a.v < b.v
            } else {
                false
            }
        } else {
            if (b.positive) {
                true
            } else {
                a.v > b.v
            }
        }
    }


    public fun abs(v: I128): u128 {
        v.v
    }

    public fun positive(v: I128): bool {
        v.positive
    }

    public fun eq(a: I128, b: I128): bool {
        a.positive == b.positive && a.v == b.v
    }


    public fun add(a: I128, b: I128): I128 {
        let n = if (a.positive) {
            if (b.positive) {
                I128 { v: a.v + b.v, positive: true }
            } else {
                I128 {
                    v: abs_sub(a.v, b.v),
                    positive: a.v >= b.v
                }
            }
        } else {
            if (b.positive) {
                I128 { v: abs_sub(a.v, b.v), positive: b.v >= a.v }
            } else {
                I128 {
                    v: a.v + b.v,
                    positive: false
                }
            }
        };
        check_v(n)
    }

    public fun sub(a: I128, b: I128): I128 {
        add(a, negative(b))
    }


    public fun mul(a: I128, b: I128): I128 {
        check_v(
            I128 {
                v: a.v * b.v,
                positive: xor(a.positive, b.positive),
            }
        )
    }

    public fun div(a: I128, b: I128): I128 {
        let div_result = a.v / b.v;
        let n = I128 {
            v: div_result,
            positive: xor(a.positive, b.positive),
        };
        let mod_result = a.v % b.v;
        if (mod_result != 0 && !n.positive) {
            n.v = n.v + 1;
        };
        check_v(n)
    }

    fun abs_sub(a: u128, b: u128): u128 {
        if (a >= b) {
            a - b
        } else {
            b - a
        }
    }

    fun xor(a: bool, b: bool): bool {
        if (a) {
            b
        } else {
            !b
        }
    }

    fun check_v(v: I128): I128 {
        // unify zero
        if (v.v == 0) {
            return zero()
        };
        if (v.positive) {
            assert!(v.v <= MAX_V, 100);
        } else {
            assert!(v.v <= MIN_V, 100);
        };
        v
    }

    #[test]
    fun test_less_than() {
        assert!(less_than(from_u128(1), from_u128(2)), 100);
        assert!(!less_than(from_u128(2), from_u128(1)), 100);
        assert!(!less_than(new(2, true), new(1, false)), 100);
    }

    #[test]
    fun test_negative() {
        assert!(negative(I128 { positive: true, v: 1 }) == I128 { positive: false, v: 1 }, 100);
        assert!(negative(I128 { positive: false, v: 1 }) == I128 { positive: true, v: 1 }, 100);
    }

    #[test]
    #[expected_failure]
    fun test_negative_err() {
        assert!(negative(I128 { positive: true, v: 1 }) == I128 { positive: true, v: 1 }, 100);
    }


    #[test]
    fun test_add() {
        assert!(add(new(1, true), new(MAX_V - 1, true)) == max(), 100);
        assert!(add(new(1, false), new(1, true)) == zero(), 100);
        assert!(add(new(1, false), new(MIN_V - 1, false)) == min(), 100);
        assert!(add(new(1, true), new(2, false)) == I128 { positive: false, v: 1 }, 100);
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
        check_v(I128 { positive: false, v: 1 });
        check_v(I128 { positive: false, v: MIN_V - 1 });
        check_v(I128 { positive: false, v: MIN_V });

        check_v(I128 { positive: true, v: 0 });
        check_v(I128 { positive: true, v: MAX_V - 1 });
        check_v(I128 { positive: true, v: MAX_V });
    }

    #[test]
    #[expected_failure]
    fun test_check_v_err() {
        check_v(I128 { positive: false, v: MIN_V + 1 });
    }

    #[test]
    #[expected_failure]
    fun test_check_v_err_2() {
        check_v(I128 { positive: true, v: MAX_V + 1 });
    }
}
