extern crate multirust;

use std::error;

pub type Error = Box<error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

mod bisect;
pub use bisect::least_satisfying;
