#[macro_use]
extern crate clap;
extern crate crypto;
extern crate protobuf;
extern crate users;
extern crate http;
extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate json;


mod error;
use error::CliError;

use futures::{Future, future};
use hyper::{Uri, Method};
use hyper::client::{Request, Client};
use tokio_core::reactor::Core;
use futures::Stream;
use json::parse;

use crypto::digest::Digest;
use crypto::sha2::Sha512;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAMESPACE: &'static str = "000001";

fn compute_address(gtin: &str) -> String {
    let mut sha = Sha512::new();
    println!("{}",gtin.to_string());
    sha.input(gtin.as_bytes());
    String::from(NAMESPACE) + &sha.result_str()[..64].to_string()
}


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
            let gtin = match matches.value_of("gtin"){
                Some(x) => x,
                None =>{
                    return Err(CliError::UserError(format!("gtin is required")))
                }
            };

            let base = "http://127.0.0.1:8008/state/".to_string();
            let address = compute_address(gtin);
            let strurl = base+&address;
            println!("{}",strurl);

            let mut core = Core::new().unwrap();

            let client = Client::new(&core.handle());

            let url : Uri = strurl.parse().unwrap();
            let mut req = Request::new(Method::Get, url);
            let work = client.request(req).and_then(|res| {
                res.body()
                    .fold(Vec::new(), |mut v, chunk| {
                        v.extend(&chunk[..]);
                        future::ok::<_, hyper::Error>(v)
                    })
                    .and_then(move |chunks| {
                        let body = String::from_utf8(chunks).unwrap();
                        future::ok(body)
                    })
            });

            let body = core.run(work)?;
            let body2 = body.to_string();
            println!("Response Body:\n{}", &body2);
            let parsed =parse(&body).unwrap();
            let mut body3 = match parsed.get("data"){
                Ok(x) => x,
                Err(err) => {
                    return Err(CliError::UserError(format!("gtin is required")))
                }
            };
            println!("{:?}", body4.as_string().to_string());
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
