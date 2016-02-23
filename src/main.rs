extern crate multirust;
extern crate rust_install;

extern crate rust_bisect;

use std::process;

use rust_bisect::{Result, cli};

fn main() {
    fn run() -> Result<i32> {
        let matches = cli::app().get_matches();
        let cfg = try!(cli::Cfg::from_matches(&matches));

        let mr_cfg =
            try!(multirust::Cfg::from_env(rust_install::notify::SharedNotifyHandler::none()));
        rust_bisect::run(&cfg, &mr_cfg)
    }

    process::exit(run().expect("something went wrong"));
}
