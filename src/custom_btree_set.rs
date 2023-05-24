use std::collections::BTreeSet;

struct Ordenable<T> {
    data: T,
    partial_cmp: fn(&T, &T) -> Option<std::cmp::Ordering>,
}

impl<T> PartialOrd for Ordenable<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.partial_cmp)(&self.data, &other.data)
    }
}

impl<T> PartialEq for Ordenable<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self.partial_cmp)(&self.data, &other.data) {
            Some(std::cmp::Ordering::Equal) => true,
            _ => false,
        }
    }
}

impl<T> Eq for Ordenable<T> {}

struct CustomBtreeSet<T: Eq + PartialEq> {
    inner_structure: BTreeSet<Ordenable<T>>,
}
