extern crate protoc_rust;
use protoc_rust::Customize;

use std::fs;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    fs::create_dir_all("src/protos").unwrap();
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/protos",
        input: &["../protos/payload.proto", "../protos/agreement.proto"],
        includes: &["../protos"],
        customize: Customize::default()
    }).expect("protoc");

    let mut file = File::create("src/protos/mod.rs").unwrap();
    file.write_all(b"pub mod payload;\n").unwrap();
    file.write_all(b"pub mod agreement;\n").unwrap();
}
