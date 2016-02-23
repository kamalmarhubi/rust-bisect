use std;
use std::fmt;
use std::ffi::OsStr;

use clap::{App, AppSettings, Arg, ArgMatches};
use rust_install::dist::ToolchainDesc;
use term;

use {NIGHTLY, Error, Nightly, Result};

pub fn app() -> App<'static, 'static> {
    fn validate_version(s: String) -> std::result::Result<(), String> {
        let ret = ToolchainDesc::from_str(&s);
        match ret {
            Some(ref desc) if desc.channel == NIGHTLY && desc.date.is_some() => Ok(()),
            Some(_) => Err(String::from("can only bisect on dated nightlies")),
            None => Err(String::from(format!("invalid version: {}", s))),
        }
    }
    App::new("rust-bisect")
        .author("Kamal Marhubi <kamal@marhubi.com>")
        .about("Find the Rust nightly that that changed some behavior")
        .setting(AppSettings::TrailingVarArg)
        .usage("rust-bisect [FLAGS] --bad <VERSION> --good <VERSION> <COMMAND> [ARGS...]")
        .arg(Arg::with_name("good")
                 .long("good")
                 .takes_value(true)
                 .value_name("VERSION")
                 .help("A known good nightly release")
                 .validator(validate_version)
                 .required(true))
        .arg(Arg::with_name("bad")
                 .long("bad")
                 .takes_value(true)
                 .value_name("VERSION")
                 .help("A known bad nightly release")
                 .validator(validate_version)
                 .required(true))
        .arg(Arg::with_name("COMMAND")
                 .index(1)
                 .help("The command to run")
                 .required(true))
        .arg(Arg::with_name("ARGS")
                 .index(2)
                 .multiple(true)
                 .help("Arguments for COMMAND"))
}

#[derive(Debug)]
pub struct Cfg<'a> {
    pub good: Nightly,
    pub bad: Nightly,
    pub cmd: &'a OsStr,
    pub args: Vec<&'a OsStr>,
}

impl<'a> Cfg<'a> {
    pub fn from_matches(matches: &'a ArgMatches<'a>) -> Result<Cfg<'a>> {
        let good = try!(matches.value_of("good").ok_or("missing arg: `good`"));
        let good: Nightly = try!(good.parse());

        let bad = try!(matches.value_of("bad").ok_or("missing arg: `bad`"));
        let bad: Nightly = try!(bad.parse());

        if bad.date < good.date {
            return Err(Error::from("`bad` must be after `good`"));
        }

        let cmd = try!(matches.value_of_os("COMMAND").ok_or("missing arg: `COMMAND`"));
        let args: Vec<_> = matches.values_of_os("ARGS")
                                  .map(|args| args.collect())
                                  .unwrap_or(Vec::new());

        Ok(Cfg {
            good: good,
            bad: bad,
            cmd: cmd,
            args: args,
        })
    }
}

pub fn display_error<E: fmt::Display>(e: E) -> Result<()> {
    use std::io::Write;
    if let Some(mut t) = term::stdout() {
        try!(t.fg(term::color::RED));
        try!(t.attr(term::Attr::Bold));

        try!(write!(t, "error: "));

        try!(t.reset());
    } else {
        print!("error: ");
    }
    println!("{}", e);

    Ok(())
}
