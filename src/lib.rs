extern crate chrono;
extern crate clap;
extern crate multirust;
extern crate rust_install;

use std::{error, fmt, str};

use chrono::NaiveDate;
use rust_install::dist::ToolchainDesc;

pub const NIGHTLY: &'static str = "nightly";

pub type Error = Box<error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

mod bisect;
pub use bisect::least_satisfying;

pub mod cli;

#[derive(Clone, Copy, Debug)]
pub struct Nightly {
    pub date: NaiveDate,
}

impl Nightly {
    pub fn to_toolchain_desc(&self) -> ToolchainDesc {
        ToolchainDesc {
            date: Some(self.date.to_string()),
            channel: String::from(NIGHTLY),
            arch: None,
            os: None,
            env: None,
        }
    }
}

impl From<NaiveDate> for Nightly {
    fn from(date: NaiveDate) -> Nightly {
        Nightly { date: date }
    }
}

impl str::FromStr for Nightly {
    type Err = Error;
    fn from_str(s: &str) -> Result<Nightly> {
        let desc = try!(ToolchainDesc::from_str(s).ok_or("invalid toolchain name"));
        if desc.channel != NIGHTLY || desc.date.is_none() {
            return Err(Error::from("not a dated nightly"));
        }
        Ok(Nightly { date: try!(desc.date.unwrap().parse()) })
    }
}

impl fmt::Display for Nightly {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-{}", NIGHTLY, self.date)
    }
}
