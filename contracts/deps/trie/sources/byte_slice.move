/// slice module for vector
module trie::byte_slice {
    use std::vector;
    public fun slice<E>(
        origin: &vector<E>,
        from: u64,
        _to: u64, // origin: [begin, end)
        start: u64,
        end: u64
    ): (&vector<E>, u64, u64) {
        (origin, from + start, from + end)
    }
    public fun to_bytes<E: copy>(origin: &vector<E>, from: u64,to: u64): vector<E> {
        let ret = vector::empty<E>();
        let i = from;
        while (i < to) {
            vector::push_back(&mut ret, *vector::borrow(origin, i));
            i=i+1;
        };
        ret
    }
}