extern crate protobuf;

use std::fs::File;
use std::io::prelude::*;
use protobuf::Message;
mod protos;
use protos::payload::{CreateAgreement, SetAgreementStatus, AgreementPayload, AgreementPayload_Action as Action};

fn main() -> std::io::Result<()>{
    let mut buffer = File::create("payload").unwrap();
    let payload: AgreementPayload = create_agreement_payload();
    let tnx_payload = payload.write_to_bytes()?;
    buffer.write_all(&tnx_payload)?;
    print!("written");
    Ok(())
}

fn create_agreement_payload() -> AgreementPayload{
    let mut create_agree = CreateAgreement::new();
    create_agree.set_name("agreement-v1-1234".to_string());
    create_agree.set_gtin("00049000015966".to_string());
    create_agree.set_price(195.00);
    create_agree.set_effectiveDate("2019-19-10".to_string());
    create_agree.set_terminationDate("2020-19-10".to_string());
    create_agree.set_unitOfQuantity("100".to_string());
    create_agree.set_paymentTerm("Net10".to_string());
    create_agree.set_originParty("SYY".to_string());
    let mut payload = AgreementPayload::new();
    payload.action = Action::CREATE_AGREEMENT;
    payload.set_create_agreement(create_agree);
    return payload;
}