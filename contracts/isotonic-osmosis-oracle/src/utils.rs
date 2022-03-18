pub fn sorted_tuple<T: PartialOrd>(elem1: T, elem2: T) -> (T, T) {
    if elem1 < elem2 {
        (elem1, elem2)
    } else {
        (elem2, elem1)
    }
}
