use clap::{App, AppSettings, Arg};
use rust_install::dist::ToolchainDesc;

use NIGHTLY;

pub fn app() -> App<'static, 'static> {
    fn validate_version(s: String) -> Result<(), String> {
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
