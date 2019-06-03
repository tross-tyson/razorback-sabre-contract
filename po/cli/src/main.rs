//TODO[Madushanka]: Should include all the operations, currently support only the creation. 

#[macro_use]
extern crate clap;
extern crate crypto;
extern crate protobuf;
extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate json;
extern crate base64;


mod error;
use error::CliError;

use futures::{Future, future};
use hyper::{Uri, Method};
use hyper::client::{Request, Client};
use tokio_core::reactor::Core;
use futures::Stream;
use json::parse;
mod protos;

use protos::po::{ POList};
use crypto::digest::Digest;
use crypto::sha2::Sha512;

const APP_NAME: &'static str = env!("CARGO_PKG_NAME");
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const NAMESPACE: &'static str = "000008";

fn compute_address(name: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(name.as_bytes());
    String::from(NAMESPACE) + &sha.result_str()[..64].to_string()
}

fn run() -> Result<(), CliError> {
    let matches = clap_app!(myapp =>
        (name: APP_NAME)
        (version: VERSION)
        (about: "Sawtooth po prgram CLI")
        (@setting SubcommandRequiredElseHelp)
        (@subcommand po =>
            (@setting SubcommandRequiredElseHelp)
            (about: "po commamds")
            (@subcommand show =>
                (about: "show a po")
                (@arg poNumber: +required "po number")
            ) 
        )
    ).get_matches();
    if let Some(matches) = matches.subcommand_matches("po") {
        if let Some(matches) = matches.subcommand_matches("show") {
            let poNumber = match matches.value_of("poNumber"){
                Some(x) => x,
                None =>{
                    return Err(CliError::UserError(format!("po number is required")))
                }
            };

            let base = "http://127.0.0.1:8008/state/".to_string();
            let address = compute_address(poNumber);
            let strurl = base+&address;
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
            let parsed =parse(&body).unwrap();
            let mut body3 = match parsed.get("data"){
                Ok(json::JsonValue::String(x)) => x,
                Ok(x) => "None",
                Err(err) => {
                    return Ok(())
                }
            };
            
            let decoded = base64::decode(body3).unwrap();
            let purchaseOrders: POList = match protobuf::parse_from_bytes(decoded.as_bytes()) {
                    Ok(purchaseOrders) => purchaseOrders,
                    Err(err) => {
                        return Err(CliError::UserError(format!("Unable to decode")))
                    } 
                };
                for po in purchaseOrders.get_purchaseOrders() {
                    if po.poNumber == poNumber {
                        println!("{:?}", poNumber);
                    }
                }
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

