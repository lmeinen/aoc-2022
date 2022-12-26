use std::ops::Add;

use regex::Regex;

#[derive(Debug)]
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

pub fn try_capture_and_parse<T>(
    re: &Regex,
    line: &str,
    group_name: &str,
    parse_fn: &dyn Fn(&str) -> T,
) -> Option<T> {
    let re_captures = re.captures(line).expect("regex failed to capture line");
    if let Some(captured_str) = re_captures.name(group_name) {
        Some(parse_fn(captured_str.as_str()))
    } else {
        None
    }
}

pub fn capture_and_parse<T>(
    re: &Regex,
    line: &str,
    group_name: &str,
    parse_fn: &dyn Fn(&str) -> T,
) -> T {
    let re_captures = re.captures(line).expect("regex failed to capture line");
    let captured_str = re_captures
        .name(group_name)
        .expect("regex didn't contain expected named capture group")
        .as_str();
    parse_fn(captured_str)
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    pub fn get_x(&self) -> &T {
        &self.x
    }

    pub fn get_y(&self) -> &T {
        &self.y
    }

    pub fn of_tuple((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

// Notice that the implementation uses the associated type `Output`.
impl<T: Add<Output = T>> Add for Point<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
