
#[macro_use]
extern crate clap;
extern crate crypto;
extern crate futures;
extern crate hyper;
extern crate protobuf;
extern crate tokio_core;
extern crate users;

mod error;
use error::CliError;


const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn run() -> Result<(), CliError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (about: "Sawtooth gtin prgram CLI")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand program =>
            (@setting SubcommandRequiredElseHelp)
            (about: "program commamds")
            (@subcommand show =>
                (about: "show a program")
                (@arg gtin: +required "gtin")
            ) 
        )
    ).get_matches();
    if let Some(matches) = matches.subcommand_matches("program") {
        if let Some(matches) = matches.subcommand_matches("show") {
            let gtin = matches.value_of("gtin");
        }
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("{}", err);
        std::process::exit(1);
    }
}

