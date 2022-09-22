/// slice module for vector
module trie::byte_slice {
    use Std::Vector;
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
        let ret = Vector::empty<E>();
        let i = from;
        while (i < to) {
            Vector::push_back(&mut ret, *Vector::borrow(origin, i));
            i=i+1;
        };
        ret
    }
}