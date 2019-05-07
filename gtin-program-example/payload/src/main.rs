use std::fs::File;
use std::io::prelude::*;

extern crate protobuf;
use protobuf::Message;
mod protos;
use protos::payload::{CreateProgramAction,ProgramPayload, ProgramPayload_Action};
fn main() -> std::io::Result<()>{
    let mut buffer = File::create("payload").unwrap();
    let payload: ProgramPayload = create_program_payload();
    let tnx_payload = payload.write_to_bytes()?;
    buffer.write_all(&tnx_payload)?;
    print!("written");
    Ok(())
}

fn create_program_payload() -> ProgramPayload{
    let mut create_program = CreateProgramAction::new();
    create_program.set_gtin("12345678".to_string());
    let mut payload = ProgramPayload::new();
    payload.action = ProgramPayload_Action::CREATE_PROGRAM;
    payload.set_create_program(create_program);
    return payload;
}