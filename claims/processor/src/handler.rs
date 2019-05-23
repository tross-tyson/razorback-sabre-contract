use protobuf;
use crypto::digest::Digest;
use crypto::sha2::Sha512;

use protos::agreement::{Agreement, AgreementStatus, AgreementStatus_Status as Status};
use protos::agreement_registry::{AgreementRegistryList, AgreementRegistry};
use protos::payload::{CreateAgreement, UpdateOrgStatus, AgreementPayload, AgreementPayload_Action as Action};



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

pub struct AgreementTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl AgreementTransactionHandler {
    pub fn new() -> ProgramTransactionHandler {
        ProgramTransactionHandler {
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
        let payload = protobuf::parse_from_bytes::<ProgramPayload>(request.get_payload())
            .map_err(|_| ApplyError::InternalError("Failed to parse payload".into()))?;
        let mut state = ProgramState::new(context);
        #[cfg(not(target_arch = "wasm32"))]
        info!(
            "{:?} {:?} {:?}",
            payload.get_action(),
            request.get_header().get_inputs(),
            request.get_header().get_outputs()
        );

        match payload.action {
            Action::CREATE_PROGRAM => create_program(payload.get_create_program(), &mut state),
            Action::UPDATE_ORG_STATUS => update_org_state(payload.get_update_org_status(), &mut state),
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