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
