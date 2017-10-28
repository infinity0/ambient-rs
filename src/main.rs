extern crate caps;
extern crate clap;
extern crate exec;

use std::str::FromStr;
use caps::{Capability, CapSet};
use clap::{Arg, App, AppSettings};
use std::string::ToString;

fn raise_capability(c: Capability) -> Result<(), caps::errors::Error> {
    // this needs to be done before the ambient set is raised
    try!(caps::raise(None, CapSet::Inheritable, c));
    try!(caps::raise(None, CapSet::Ambient, c));
    Ok(())
}

fn main() {
    let matches = App::new("Set Ambient Capabilities")
        .version("0.0")
        .author("Ximin Luo <infinity0@pwned.gg>")
        .setting(AppSettings::TrailingVarArg)
        .setting(AppSettings::AllowLeadingHyphen)
        .arg(Arg::with_name("quiet")
             .short("q")
             .long("quiet")
             .help("Suppress non-error messages")
             .takes_value(false))
        .arg(Arg::with_name("caps")
             .short("c")
             .long("capability")
             .help("Set an ambient capability.")
             .takes_value(true)
             .value_name("CAP")
             .validator(|s| Capability::from_str(&s).map(|_| ()).map_err(|e| e.to_string()))
             .use_delimiter(true)
             .number_of_values(1)
             .multiple(true))
        .arg(Arg::with_name("args")
             .help("Program to run with the given capabilities.")
             .multiple(true))
        .get_matches();

    let caps: Vec<_> = matches.values_of("caps").unwrap_or(clap::Values::default()).collect();
    let args: Vec<_> = matches.values_of("args").unwrap_or(clap::Values::default()).collect();
    let quiet = matches.occurrences_of("quiet") > 0;

    for cap in caps {
        let capv = Capability::from_str(&cap.to_string()).unwrap();
        match raise_capability(capv) {
            Ok(_) => if !quiet {
                eprintln!("Raised in inheritable and ambient capsets: {} ({:?})",
                          cap, capv)
            },
            Err(e) =>
                eprintln!("Error raising cap \"{}\": {}\n  \
                           try running: p={:?}; sudo setcap \"$(sudo getcap \"$p\" | sed -n 's/^.*=\\s*//p') {:?}+eip\" \"$p\"",
                          cap, e, std::env::current_exe().unwrap(), capv),
        }
    }

    let cur = caps::read(None, CapSet::Effective).unwrap();
    if !quiet {
        eprintln!("Current effective caps: {:?}.", cur)
    }

    if args.len() < 1 {
        panic!("Usage: ambient [-c caps] prog [args]");
    }

    eprintln!("error execvp: {}", exec::execvp(&args[0], &args));
}
