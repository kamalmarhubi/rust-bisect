extern crate multirust;

use std::error;

pub type Error = Box<error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

/// Finds the index of the least item in `slice` for which the `predicate` holds.
// Adapted from core::slice::binary_search_by().
pub fn least_satisfying<T, P>(slice: &[T], mut predicate: P) -> usize
    where P: FnMut(&T) -> bool
{
    // TODO: assert if fails and succedds at the ends?
    // TODO: make it return Option<usize>, in case predicate holds nowhere.
    let mut base = 0usize;
    let mut s = slice;

    loop {
        let (head, tail) = s.split_at(s.len() >> 1);
        if tail.is_empty() {
            return base + head.len();
        }
        if predicate(&tail[0]) {
            s = head;
        } else {
            base += head.len() + 1;
            s = &tail[1..];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bisect() {
        let s = &[0, 3, 7, 10, 33, 169, 222, 223];
        assert_eq!(0, least_satisfying(s, |&x| x >= 0));
        assert_eq!(5, least_satisfying(s, |&x| x >= 50));
        assert_eq!(1, least_satisfying(s, |&x| x >= 3));
        assert_eq!(2, least_satisfying(s, |&x| x > 3));
    }
}
