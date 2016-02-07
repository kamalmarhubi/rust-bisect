#[macro_use(shared_ntfy)]
extern crate rust_install;

extern crate multirust;

use multirust::{Cfg, Result, Notification};

// Copied from multirust-rs.
fn set_globals() -> Result<Cfg> {
    // Base config
    let verbose = false;
    Cfg::from_env(shared_ntfy!(move |n: Notification| {
        use multirust::notify::NotificationLevel::*;
        match n.level() {
            Verbose => {
                if verbose {
                    println!("{}", n);
                }
            }
            _ => {
                println!("{}", n);
            }
        }
    }))
}

fn main() {
    let cfg = set_globals().expect("set_globals");
    println!("{:?}", cfg.multirust_dir);
}
