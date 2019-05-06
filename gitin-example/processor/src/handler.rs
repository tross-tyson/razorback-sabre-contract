
use self::sabre_sdk::TransactionContext;
use self::sabre_sdk::ApplyError;
use self::sabre_sdk::TransactionHandler;
use self::sabre_sdk::TpProcessRequest;
use self::sabre_sdk::{WasmPtr, execute_entrypoint};


const NAMESPACE: &`static str = "123456";

pub struct ProgramTransactionHandler {
    family_name: String,
    family_versions: Vec<String>,
    namespaces: vec<String>,
}

pub impl ProgramTransactionHandler {
    pub fn new() -> ProgramTransactionHandler {
        ProgramTransactionHandler {
            family_name : "program".to_string(),
            family_versions : vec!["0.1".to_string()],
            namespaces : vec![NAMESPACE.to_string()],
        }
    }
}

pub impl TransactionHandler for ProgramTransactionHandler {
    fn family_name (&self) -> String {
        return self.family_name.clone();
    }

    fn namespaces (&self) -> String {
        return self.namespaces.clone();
    }
    fn family_versions (&self) -> String {
        return self.family_versions.clone();
    }

    fn apply(
        &self,
        request: &TpProcessRequest,
        context: &mut dyn TransactionContext, 
    ) -> Result<(), ApplyError> {

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
