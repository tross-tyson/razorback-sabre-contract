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

use protos::agreement::{Agreement, AgreementStatus, AgreementStatus_Status, AgreementList};
use protos::payload::{CreateAgreement, SetAgreementStatus, AgreementPayload, AgreementPayload_Action as Action};

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

const AGREEMENT_NAMESPACE: &'static str = "000004";

fn compute_agreement_address(name: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(name.as_bytes());
    String::from(AGREEMENT_NAMESPACE) + &sha.result_str()[..64].to_string()
}

// State representation of an agreemenent
pub struct AgreementState<'a> {
    context : &'a mut TransactionContext,
}

impl <'a> AgreementState<'a> {
    pub fn new(context: &'a mut TransactionContext) -> AgreementState {
        AgreementState { context: context }
    }
    pub fn get_agreement(&mut self, name : &str) -> Result <Option<Agreement>, ApplyError> {
        let address = compute_agreement_address(name);
        let a = self.context.get_state_entry(&address)?;
        match a {
            Some(packed) => {
                let agreements: AgreementList = match protobuf::parse_from_bytes(packed.as_slice()) {
                    Ok(agreements) => agreements,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize agreement list: {:?}",
                            err,
                        )))
                    } 
                };
                for agreement in agreements.get_agreements() {
                    if agreement.name == name {
                        return Ok(Some(agreement.clone()));
                    }
                }
                Ok (None)
            }
            None => Ok(None),
        }
    }
    pub fn set_agreement(&mut self, name: &str, new_agreement: Agreement) -> Result<(), ApplyError> {
        let address = compute_agreement_address(name);
        let d = self.context.get_state_entry(&address);
        let mut agreement_list = match d {
            Ok(Some(packed)) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(agreements) => agreements,
                Err(err) =>{
                return Err(ApplyError::InternalError(String::from(
                    "Cannot decode the agreement list",
                )))
            }
            },
            Ok(None) => AgreementList::new(),
            Err(err) =>{
                return Err(ApplyError::InternalError(String::from(
                    "Cannot decode the agreement list",
                )))
            }
        };

        let agreements = agreement_list.get_agreements().to_vec();
        let mut index = None;
        let mut count = 0;
        for agree in agreements.clone() {
            if agree.name == name {
               index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                agreement_list.agreements.remove(x);
            }
            None => (),
        };
        agreement_list.agreements.push(new_agreement);
        let serialized = match protobuf::Message::write_to_bytes(&agreement_list) {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize agreement list",
                )))
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }
}

fn create_agreement(
    payload: &CreateAgreement,
    state: &mut AgreementState
) -> Result<(), ApplyError> {
    match state.get_agreement(payload.get_name()) {
        Ok(None) => (),
        Ok(Some(_)) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Agreement already exists: {}",
                payload.get_gtin(),
            )))
        }
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    }  
    let mut agreement = Agreement::new();
    agreement.set_name(payload.get_name().to_string());
    agreement.set_gtin(payload.get_gtin().to_string());
    agreement.set_price(payload.get_price());
    agreement.set_effectiveDate(payload.get_effectiveDate().to_string());
    agreement.set_terminationDate(payload.get_terminationDate().to_string());
    agreement.set_unitOfQuantity(payload.get_unitOfQuantity().to_string());
    agreement.set_paymentTerm(payload.get_paymentTerm().to_string());
    agreement.set_originParty(payload.get_originParty().to_string());

    let mut agreementStatus = AgreementStatus::new();
    agreementStatus.set_party(payload.get_originParty().to_string());
    agreementStatus.set_status(AgreementStatus_Status::INITIATED);
    agreement.agreementStatus.push(agreementStatus);

    state.set_agreement(payload.get_name(), agreement)
        .map_err(|e| ApplyError::InternalError(format!("Failed to create agreement: {:?}",e)))
}


fn set_agreement_status(
    payload: &SetAgreementStatus,
    state: &mut AgreementState
) -> Result<(), ApplyError> {
    let mut agreement = match state.get_agreement(payload.get_name()){
        Ok(None) => {
            return Err(ApplyError:: InvalidTransaction(format!(
                "Agreement does not exists : {}", 
                payload.get_name(),
            )))
        }
        Ok (Some(agreement)) => agreement,
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };

    let mut agreementStatus = AgreementStatus::new();
    agreementStatus.set_party(payload.get_party().to_string());
    agreementStatus.set_status(AgreementStatus_Status::AGREED);
    agreement.agreementStatus.push(agreementStatus);
    state.set_agreement(payload.get_name(), agreement)
        .map_err(|e| ApplyError::InternalError(format!("Failed to update the agreement: {:?}",e)))

}

pub struct AgreementTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl AgreementTransactionHandler {
    pub fn new() -> AgreementTransactionHandler {
        AgreementTransactionHandler {
            family_name : "agreement".to_string(),
            family_versions : vec!["0.1".to_string()],
            namespaces : vec![AGREEMENT_NAMESPACE.to_string()],
        }
    }
}

impl TransactionHandler for AgreementTransactionHandler {
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
        let payload = protobuf::parse_from_bytes::<AgreementPayload>(request.get_payload())
            .map_err(|_| ApplyError::InternalError("Failed to parse agreement payload".into()))?;
        let mut state = AgreementState::new(context);
        #[cfg(not(target_arch = "wasm32"))]
        info!(
            "{:?} {:?} {:?}",
            payload.get_action(),
            request.get_header().get_inputs(),
            request.get_header().get_outputs()
        );

        match payload.action {
            Action::CREATE_AGREEMENT => create_agreement(payload.get_create_agreement(), &mut state),
            Action::SET_AGREEMENT_STATUS => set_agreement_status(payload.get_set_agreement_status(), &mut state),
            _ => Err(ApplyError::InvalidTransaction("Invalid action".into())),
        }

    }
}

#[cfg(target_arch = "wasm32")]
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    let handler = AgreementTransactionHandler::new();
    match handler.apply(request, context) {
        Ok(_) => Ok(true),
        Err(err) => Err(err),
    }   
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn entrypoint(payload: WasmPtr, signer: WasmPtr, signature: WasmPtr) -> i32 {
    execute_entrypoint(payload, signer, signature, apply)
}