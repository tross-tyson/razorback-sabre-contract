extern crate protobuf;

use std::fs::File;
use std::io::prelude::*;
use protobuf::Message;
mod protos;
use protos::payload::{CreatePO, ReceivePO, POPayload, POPayload_Action as Action};
use protos::po::{PO, Item, POStatus, POStatus_OrderStatus, POList};

fn main() -> std::io::Result<()>{
    let mut buffer = File::create("payload").unwrap();
    let payload: POPayload = create_po_payload();
    let tnx_payload = payload.write_to_bytes()?;
    buffer.write_all(&tnx_payload)?;
    print!("written");
    Ok(())
}

fn create_po_payload() -> POPayload{
    let mut create_po = CreatePO::new();
    create_po.set_poNumber("po-v1-1234".to_string());
    create_po.set_poDate("2020-19-7".to_string());
    create_po.set_totalAmount(1345.00);
    create_po.set_shipDate("2019-19-14".to_string());
    create_po.set_paymentDate("2020-19-10".to_string());
    create_po.set_originParty("SYY".to_string());
    
    let mut item1 =  Item::new();
    item1.set_gtin("00099345897431".to_string());
    item1.set_quantity(100);
    item1.set_price(19000.00);
    item1.set_agreementID("agreement-v1-1234".to_string());
    item1.set_carrierName("SYY".to_string());
    create_po.items.push(item1);

    let mut item2 =  Item::new();
    item2.set_gtin("00099345897465".to_string());
    item2.set_quantity(89);
    item2.set_price(56000.00);
    item2.set_agreementID("agreement-v2-1234".to_string());
    item2.set_carrierName("DDD".to_string());
    create_po.items.push(item2);

    let mut payload = POPayload::new();
    payload.action = Action::CREATE_PO;
    payload.set_create_po(create_po);
    return payload;
}