
// Copyright 2019 Sysco Labs
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use protobuf;
use crypto::digest::Digest;
use crypto::sha2::Sha512;

use protos::po::{PO, Item, POStatus, POStatus_OrderStatus, POList};
use protos::payload::{CreatePO, ReceivePO, POPayload, POPayload_Action as Action};

// Encapuslate all the biz logic related to 'Agreements'

cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        use sabre_sdk::ApplyError;
        use sabre_sdk::TransactionContext;
        use sabre_sdk::TransactionHandler;
        use sabre_sdk::TpProcessRequest;
        use sabre_sdk::{WasmPtr, execute_entrypoint};
    } else {
        use sawtooth_sdk::processor::handler::ApplyError;
        use sawtooth_sdk::processor::handler::TransactionContext;
        use sawtooth_sdk::processor::handler::TransactionHandler;
        use sawtooth_sdk::messages::processor::TpProcessRequest;
    }
}
const PO_NAMESPACE: &'static str = "000008";

//TODO - Move all address calc logic to a reusable Util lib
fn compute_po_address(name: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(name.as_bytes());
    String::from(PO_NAMESPACE) + &sha.result_str()[..64].to_string()
}

// State representation of a purchase order
pub struct POState<'a> {
    context : &'a mut TransactionContext,
}

impl <'a> POState<'a> {
    pub fn new(context: &'a mut TransactionContext) -> POState {
        POState { context: context }
    }
    pub fn get_po(&mut self, poNumber : &str) -> Result <Option<PO>, ApplyError> {
        let address = compute_po_address(poNumber);
        let a = self.context.get_state_entry(&address)?;
        match a {
            Some(packed) => {
                let purchaseOrders: POList = match protobuf::parse_from_bytes(packed.as_slice()) {
                    Ok(purchaseOrders) => purchaseOrders,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize po list: {:?}",
                            err,
                        )))
                    } 
                };
                for po in purchaseOrders.get_purchaseOrders() {
                    if po.poNumber == poNumber {
                        return Ok(Some(po.clone()));
                    }
                }
                Ok (None)
            }
            None => Ok(None),
        }
    }
    pub fn set_po(&mut self, poNumber: &str, new_po: PO) -> Result<(), ApplyError> {
        let address = compute_po_address(poNumber);
        let d = self.context.get_state_entry(&address);
        let mut purchaseOrder_list = match d {
            Ok(Some(packed)) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(purchaseOrders) => purchaseOrders,
                Err(err) =>{
                return Err(ApplyError::InternalError(String::from(
                    "Cannot decode the po list",
                )))
            }
            },
            Ok(None) => POList::new(),
            Err(err) =>{
                return Err(ApplyError::InternalError(String::from(
                    "Cannot decode the po list",
                )))
            }
        };

        let purchaseOrders = purchaseOrder_list.get_purchaseOrders().to_vec();
        let mut index = None;
        let mut count = 0;
        for po in purchaseOrders.clone() {
            if po.poNumber == poNumber {
               index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                purchaseOrder_list.purchaseOrders.remove(x);
            }
            None => (),
        };
        purchaseOrder_list.purchaseOrders.push(new_po);
        let serialized = match protobuf::Message::write_to_bytes(&purchaseOrder_list) {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize po list",
                )))
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }
}

fn create_po(
    payload: &CreatePO,
    state: &mut POState
) -> Result<(), ApplyError> {
    match state.get_po(payload.get_poNumber()) {
        Ok(None) => (),
        Ok(Some(_)) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "PO already exists: {}",
                payload.get_poNumber(),
            )))
        }
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    }  
    let mut po = PO::new();
    po.set_poNumber(payload.get_poNumber().to_string());
    po.set_poDate(payload.get_poDate().to_string());
    po.set_totalAmount(payload.get_totalAmount());
    po.set_shipDate(payload.get_shipDate().to_string());
    po.set_paymentDate(payload.get_paymentDate().to_string());
    po.set_originParty(payload.get_originParty().to_string());
  //  for item in payload.get_items(){
  //      po.items.push(item);
  //  }
    let mut poStatus = POStatus::new();
    poStatus.set_party(payload.get_originParty().to_string());
    poStatus.set_status(POStatus_OrderStatus::ORDERED);
    po.orderStatusHistory.push(poStatus);

    state.set_po(payload.get_poNumber(), po)
        .map_err(|e| ApplyError::InternalError(format!("Failed to create po: {:?}",e)))
}

fn receive_po(
    payload: &ReceivePO,
    state: &mut POState
) -> Result<(), ApplyError> {

    let mut po = match state.get_po(payload.get_poNumber()){
        Ok(None) => {
            return Err(ApplyError:: InvalidTransaction(format!(
                "PO does not exists : {}", 
                payload.get_poNumber(),
            )))
        }
        Ok (Some(po)) => po,
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    }; 

    let mut poStatus = POStatus::new();
    poStatus.set_party(payload.get_party().to_string());
    poStatus.set_date(payload.get_date().to_string());
    poStatus.set_status(POStatus_OrderStatus::RECEIVED);
    po.orderStatusHistory.push(poStatus);

    state.set_po(payload.get_poNumber(), po)
        .map_err(|e| ApplyError::InternalError(format!("Failed to change the status of the po tp received: {:?}",e)))
}


// Logic related to transaction handling 
pub struct POTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl POTransactionHandler {
    pub fn new() -> POTransactionHandler {
        POTransactionHandler {
            family_name : "purchase-order".to_string(),
            family_versions : vec!["0.1".to_string()],
            namespaces : vec![PO_NAMESPACE.to_string()],
        }
    }
}

impl TransactionHandler for POTransactionHandler {
    fn family_name (&self) -> String {
        return self.family_name.clone();
    }

    fn namespaces (&self) -> Vec<String> {
        return self.namespaces.clone();
    }
    fn family_versions (&self) -> Vec<String> {
        return self.family_versions.clone();
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext, 
    ) -> Result<(), ApplyError> {
        let payload = protobuf::parse_from_bytes::<POPayload>(request.get_payload())
            .map_err(|_| ApplyError::InternalError("Failed to parse po payload".into()))?;
        let mut state = POState::new(context);
        #[cfg(not(target_arch = "wasm32"))]
        info!(
            "{:?} {:?} {:?}",
            payload.get_action(),
            request.get_header().get_inputs(),
            request.get_header().get_outputs()
        );

        match payload.action {
            Action::CREATE_PO => create_po(payload.get_create_po(), &mut state),
            Action::RECEIVE_PO => receive_po(payload.get_receive_po(), &mut state),
            _ => Err(ApplyError::InvalidTransaction("Invalid action".into())),
        }

    }
}

//Initiating trasnaction
#[cfg(target_arch = "wasm32")]
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = POTransactionHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => Err(err),
    }   
}

// Entry point of the PO smart contract for WASM architectures
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn entrypoint(payload: WasmPtr, signer: WasmPtr, signature: WasmPtr) -> i32 {
    execute_entrypoint(payload, signer, signature, apply)
}