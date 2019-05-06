
use self::sabre_sdk::TransactionContext;
use self::sabre_sdk::ApplyError;
use self::sabre_sdk::TransactionHandler;
use self::sabre_sdk::TpProcessRequest;
use self::sabre_sdk::{WasmPtr, execute_entrypoint};


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
