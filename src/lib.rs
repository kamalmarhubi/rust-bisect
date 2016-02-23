extern crate chrono;
extern crate clap;
extern crate multirust;
extern crate rust_install;

use std::error;

use chrono::NaiveDate;

pub const NIGHTLY: &'static str = "nightly";

pub type Error = Box<error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

mod bisect;
pub use bisect::least_satisfying;

pub mod cli;

pub fn nightly(date: NaiveDate) -> String {
    format!("{}-{}", NIGHTLY, date)
}
