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

use protos::payload::{CreateProgramAction, UpdateOrgStatus, ProgramPayload, ProgramPayload_Action as Action};
use protos::state::{ProgramList, Program, OrgStatus};


const NAMESPACE: &'static str = "123456";


fn compute_address(gtin: &str) -> String {
    let mut sha = Sha512::new();
    sha.input(gtin.as_bytes());
    String::from(NAMESPACE) + &sha.result_str()[..64].to_string()
}

pub struct ProgramState<'a> {
    context : &'a mut TransactionContext,
}

impl <'a> ProgramState<'a> {
    pub fn new(context: &'a mut TransactionContext) -> ProgramState {
        ProgramState { context: context }
    }
    pub fn get_program(&mut self, gtin : &str) -> Result <Option<Program>, ApplyError> {
        let address = compute_address(gtin);
        let d = self.context.get_state_entry(&address)?;
        match d {
            Some(packed) => {
                let progs: ProgramList = match protobuf::parse_from_bytes(packed.as_slice()) {
                    Ok(progs) => progs,
                    Err(err) => {
                        return Err(ApplyError::InternalError(format!(
                            "Cannot deserialize Program list: {:?}",
                            err,
                        )))
                    } 
                };
                for prog in progs.get_programs() {
                    if prog.gtin == gtin {
                        return Ok(Some(prog.clone()));
                    }
                }
                Ok (None)
            }
            None => Ok(None),
        }
    }
    pub fn set_program(&mut self, gtin: &str, new_program: Program) -> Result<(), ApplyError> {
        let address = compute_address(gtin);
        let d = self.context.get_state_entry(&address);
        let mut program_list = match d {
            Ok(Some(packed)) => match protobuf::parse_from_bytes(packed.as_slice()) {
                Ok(progs) => progs,
                Err(err) =>{
                return Err(ApplyError::InternalError(String::from(
                    "Cannot decode the program list",
                )))
            }
            },
            Ok(None) => ProgramList::new(),
            Err(err) =>{
                return Err(ApplyError::InternalError(String::from(
                    "Cannot decode the program list",
                )))
            }
        };

        let programs = program_list.get_programs().to_vec();
        let mut index = None;
        let mut count = 0;
        for program in programs.clone() {
            if program.gtin == gtin {
               index = Some(count);
                break;
            }
            count = count + 1;
        }

        match index {
            Some(x) => {
                program_list.programs.remove(x);
            }
            None => (),
        };
        program_list.programs.push(new_program);
        let serialized = match protobuf::Message::write_to_bytes(&program_list) {
            Ok(serialized) => serialized,
            Err(_) => {
                return Err(ApplyError::InternalError(String::from(
                    "Cannot serialize program list",
                )))
            }
        };
        self.context
            .set_state_entry(address, serialized)
            .map_err(|err| ApplyError::InternalError(format!("{}", err)))?;
        Ok(())
    }
}


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

fn create_program(
    payload: &CreateProgramAction,
    state: &mut ProgramState
) -> Result<(), ApplyError> {
    match state.get_program(payload.get_gtin()) {
        Ok(None) => (),
        Ok(Some(_)) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Program already exists: {}",
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
    let mut program = Program::new();
    program.set_gtin(payload.get_gtin().to_string());
    state
        .set_program(payload.get_gtin(), program)
        .map_err(|e| ApplyError::InternalError(format!("Failed to create program: {:?}",e)))
}

fn update_org_state(
    payload: &UpdateOrgStatus,
    state: &mut ProgramState
) -> Result<(), ApplyError> {
    let mut program = match state.get_program(payload.get_gtin()){
        Ok(None) => {
            return Err(ApplyError:: InvalidTransaction(format!(
                "Program does not exists : {}", 
                payload.get_gtin(),
            )))
        }
        Ok (Some(program)) => program,
        Err(err) => {
            return Err(ApplyError::InvalidTransaction(format!(
                "Failed to retrieve state: {}",
                err,
            )))
        }
    };
    let statuses = program.get_org_status().to_vec();
    let mut index = None;
    let mut count = 0;
    for status in statuses.clone() {
        if status.get_org_name() == payload.get_org_status().clone().get_org_name() {
            index = Some(count);
            break;
        }
        count = count + 1;
    }

    match index {
        Some(x) => {
            program.org_status.remove(x);
        }
        None => (),
    };
    program.org_status.push(payload.get_org_status().clone());
    state
        .set_program(payload.get_gtin(), program)
        .map_err(|e| ApplyError::InternalError(format!("Failed to update the program: {:?}",e)))

}

// Sabre apply must return a bool
#[cfg(target_arch = "wasm32")]
fn apply(
    request: &TpProcessRequest,
    context: &mut dyn TransactionContext,
) -> Result<bool, ApplyError> {
    Ok(true)    
}
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub unsafe fn entrypoint(payload: WasmPtr, signer: WasmPtr, signature: WasmPtr) -> i32 {
    execute_entrypoint(payload, signer, signature, apply)
}
