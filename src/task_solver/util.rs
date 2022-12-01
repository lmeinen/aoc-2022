/// A sorted list of constant size
pub struct SortedList<T: PartialOrd> {
    capacity: usize,
    items: Vec<T>,
}

impl<T: PartialOrd> SortedList<T> {
    pub fn new(capacity: usize) -> SortedList<T> {
        SortedList {
            capacity,
            items: Vec::with_capacity(capacity + 1),
        }
    }

    /// inserts an item into the list if possible
    pub fn insert(&mut self, item: T) {
        let mut insert_i = 0;
        for (i, curr) in self.items.iter().enumerate() {
            if &item > curr {
                break;
            } else {
                insert_i = i + 1;
            }
        }

        if insert_i < self.capacity {
            self.items.insert(insert_i, item);
            self.items.truncate(self.capacity);
        }
    }

    pub fn fold<U>(&self, init: U, f: &dyn Fn(&T, U) -> U) -> U {
        let mut acc = init;
        for curr in self.items.iter() {
            acc = f(curr, acc);
        }
        acc
    }
}
