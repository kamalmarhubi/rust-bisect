extern crate chrono;
extern crate multirust;
extern crate semver;

use std::{error, str};
use std::ffi::OsStr;
use std::fmt::{self, Display};
use std::ops::Range;

use chrono::NaiveDate;
use multirust::Toolchain;
use semver::Version;

pub type Error = Box<error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

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
    where P: FnMut(i32) -> Option<bool>
{
    if r.is_empty() {
        return None;
    }

    loop {
        if r.is_empty() {
            return match predicate(r.end) {
                Some(true) => Some(r.end),
                _ => None,
            };
        }
        if r.is_singleton() {
            return if let Some(true) = predicate(r.start) {
                Some(r.start)
            } else if let Some(true) = predicate(r.end) {
                // TODO test we don't go out of range because of this clause
                Some(r.end)
            } else {
                None
            };
        }
        let mut mid = r.start + (r.end - r.start) / 2;

        let mut mid_res;
        loop {
            mid_res = predicate(mid);
            if mid_res.is_some() {
                break;
            }
            // TODO: ensure we're in range, possibly searching up and down to remain as close as
            // possible to intended value
            mid = mid + 1;
        }

        r = if mid_res.expect("should be ok at this point") {
            Range {
                start: r.start,
                end: mid,
            }
        } else {
            Range {
                start: mid + 1,
                end: r.end,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bisect() {
        assert_eq!(None, bisect(0..0, |x| Some(x >= 0)));
        assert_eq!(Some(50), bisect(0..100, |x| Some(x >= 50)));
        assert_eq!(None, bisect(0..100, |x| Some(x >= 1000)));
    }
}

#[derive(Clone, Debug)]
pub enum ToolchainSpec {
    Stable(Version),
    Nightly(NaiveDate),
}
use ToolchainSpec::*;

impl str::FromStr for ToolchainSpec {
    type Err = Error;

    fn from_str(s: &str) -> Result<ToolchainSpec> {
        const NIGHTLY: &'static str = "nightly-";
        if s.starts_with(NIGHTLY) {
            return Ok(Nightly(try!(s[NIGHTLY.len()..].parse())));
        } else {

        }
        unimplemented!();
    }

}

impl From<NaiveDate> for ToolchainSpec {
    fn from(date: NaiveDate) -> ToolchainSpec {
        Nightly(date)
    }
}

impl From<Version> for ToolchainSpec {
    fn from(v: Version) -> ToolchainSpec {
        Stable(v)
    }
}

impl Display for ToolchainSpec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Stable(ref v) => Display::fmt(v, f),
            Nightly(date) => write!(f, "nightly-{}", date),
        }
    }
}

pub struct Cmd<'a> {
    program: &'a OsStr,
    args: &'a [&'a OsStr],
}

impl<'a> Cmd<'a> {
    pub fn from(command: &'a [&'a OsStr]) -> Cmd<'a> {
        let program = command[0];
        let args = &command[1..];

        Cmd {
            program: program,
            args: args,
        }
    }

    pub fn succeeds_with<'b>(&self, toolchain: &Toolchain<'b>) -> Result<bool> {
        let mut cmd = try!(toolchain.create_command(&self.program));
        cmd.args(self.args);
        let status = try!(cmd.status());
        Ok(status.success())
    }
}
