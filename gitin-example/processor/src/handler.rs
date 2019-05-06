
use protobuf;
use crypto::digest::Digest;
use crypto::sha2::Sha512;

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


const NAMESPACE: &'static str = "123456";

pub struct ProgramTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: Vec<String>,
}

impl ProgramTransactionHandler {
    pub fn new() -> ProgramTransactionHandler {
        ProgramTransactionHandler {
            family_name : "program".to_string(),
            family_versions : vec!["0.1".to_string()],
            namespaces : vec![NAMESPACE.to_string()],
        }
    }
}

impl TransactionHandler for ProgramTransactionHandler {
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
        Ok(())
    }
}
// Sabre apply must return a bool
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    Ok(true)    
}
#[no_mangle]
pub unsafe fn entrypoint(payload: WasmPtr, signer: WasmPtr, signature: WasmPtr) -> i32 {
    execute_entrypoint(payload, signer, signature, apply)
}
