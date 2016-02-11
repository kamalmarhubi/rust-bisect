use std::ops::Range;

pub trait RangeExt {
    fn is_empty(&self) -> bool;
    fn is_singleton(&self) -> bool;
}

impl RangeExt for Range<i32> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.start >= self.end
    }
    #[inline]
    fn is_singleton(&self) -> bool {
        self.start + 1 == self.end
    }
}

/// Finds least item in `r` for which the `predicate` holds.
pub fn bisect<P>(mut r: Range<i32>, mut predicate: P) -> Option<i32>
    where P: FnMut(i32) -> bool
{
    if r.is_empty() {
        return None;
    }

    loop {
        if r.is_empty() {
            return match predicate(r.end) {
                true => Some(r.end),
                false => None,
            };
        }
        if r.is_singleton() {
            return if predicate(r.start) {
                Some(r.start)
            } else {
                None
            };
        }
        let mid = r.start + (r.end - r.start) / 2;
        if predicate(mid) {
            r = Range {
                start: r.start,
                end: mid,
            };
        } else {
            r = Range {
                start: mid + 1,
                end: r.end,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bisect() {
        assert_eq!(None, bisect(0..0, |x| x >= 0));
        assert_eq!(Some(50), bisect(0..100, |x| x >= 50));
        assert_eq!(None, bisect(0..100, |x| x >= 1000));
    }
}
