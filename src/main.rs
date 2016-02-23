extern crate chrono;
extern crate hyper;
extern crate libc;
extern crate multirust;
extern crate rust_install;

extern crate rust_bisect;

use std::process;

use chrono::NaiveDate;
use hyper::client::Client;

use rust_bisect::cli;
use rust_bisect::{Nightly, Result, least_satisfying};

fn list_available_nightlies(dist_root: &str,
                            from: NaiveDate,
                            to: NaiveDate)
                            -> Result<Vec<Nightly>> {
    assert!(from < to, "`from` must be less than `to`");
    println!("finding available nightlies between {} and {}", from, to);
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
    println!("found {} nightlies", nightlies.len());
    Ok(nightlies)
}

fn run_rust_bisect() -> Result<i32> {
    let matches = cli::app().get_matches();

    let cfg = try!(multirust::Cfg::from_env(rust_install::notify::SharedNotifyHandler::none()));

    let good: Nightly = matches.value_of("good")
                               .expect("clap didn't respect required arg `good`")
                               .parse()
                               .expect("clap validator misbehaved for `good`");
    let bad: Nightly = matches.value_of("bad")
                              .expect("clap didn't respect required arg `bad`")
                              .parse()
                              .expect("clap validator misbehaved for `bad`");

    let cmd = matches.value_of_os("COMMAND")
                     .expect("clap didn't respect required arg `COMMAND`");
    let args: Vec<_> = matches.values_of_os("ARGS")
                              .map(|args| args.collect())
                              .unwrap_or(Vec::new());

    let nightlies = try!(list_available_nightlies(&*cfg.dist_root_url, good.date, bad.date));
    println!("bisecting across {} nightlies (about {} steps)",
             nightlies.len(),
             nightlies.len().next_power_of_two().trailing_zeros());

    let idx = least_satisfying(&nightlies[..], |nightly| {
        println!("testing with {}", nightly);

        let toolchain = cfg.get_toolchain(&nightly.to_string(), false)
                           .expect("could not get toolchain");
        toolchain.install_from_dist_if_not_installed().expect("could not install toolchain");

        let mut cmd = toolchain.create_command(cmd).expect("could not create command");
        cmd.args(&args);
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

fn main() {
    process::exit(run_rust_bisect().expect("something went wrong"));
}
