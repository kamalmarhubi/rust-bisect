#[macro_use(shared_ntfy)]
extern crate rust_install;

extern crate chrono;
extern crate clap;
extern crate hyper;
extern crate libc;
extern crate multirust;
extern crate term;

use std::{error, fmt, str};
use std::ffi::OsStr;

use chrono::NaiveDate;
use hyper::client::Client;
use rust_install::dist::ToolchainDesc;

const NIGHTLY: &'static str = "nightly";

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
    fn to_toolchain_desc(&self) -> ToolchainDesc {
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

#[derive(Debug)]
pub struct Cfg<'a> {
    pub good: Nightly,
    pub bad: Nightly,
    pub cmd: &'a OsStr,
    pub args: Vec<&'a OsStr>,
}

fn list_available_nightlies(dist_root: &str,
                            from: NaiveDate,
                            to: NaiveDate)
                            -> Result<Vec<Nightly>> {
    assert!(from < to, "`from` must be less than `to`");
    let client = Client::new();
    let mut nightlies = Vec::with_capacity((to - from).num_days() as usize);
    let mut date = from;
    while date < to {
        let nightly = Nightly::from(date);
        let manifest_url = nightly.to_toolchain_desc().manifest_url(dist_root);
        let resp = try!(client.head(&manifest_url).send());
        // TODO: ensure failures are 404
        if resp.status.is_success() {
            nightlies.push(nightly);
        }
        date = date.succ();
    }
    Ok(nightlies)
}

pub fn run<'a>(cfg: &'a Cfg, mr_cfg: &multirust::Cfg) -> Result<i32> {

    println!("finding available nightlies between {} and {}",
             cfg.good,
             cfg.bad);
    let nightlies = try!(list_available_nightlies(&*mr_cfg.dist_root_url,
                                                  cfg.good.date,
                                                  cfg.bad.date));
    if nightlies.is_empty() {
        try!(cli::display_error(format!("no nightlies found between {} and {}",
                                        cfg.good,
                                        cfg.bad)));
        return Ok(libc::EXIT_FAILURE);
    }

    println!("bisecting across {} nightlies (about {} steps)",
             nightlies.len(),
             nightlies.len().next_power_of_two().trailing_zeros());

    let idx = least_satisfying(&nightlies[..], |nightly| {
        println!("testing with {}", nightly);

        let toolchain = mr_cfg.get_toolchain(&nightly.to_string(), false)
                              .expect("could not get toolchain");
        toolchain.install_from_dist_if_not_installed().expect("could not install toolchain");

        let mut cmd = toolchain.create_command(cfg.cmd).expect("could not create command");
        cmd.args(&cfg.args);
        let res = cmd.status().expect("could not run command").success();

        println!("command {} with {}",
                 if res {
                     "succeeded"
                 } else {
                     "failed"
                 },
                 nightly);
        !res
    });

    println!("{} is the first failing nightly", nightlies[idx]);
    Ok(libc::EXIT_SUCCESS)
}
