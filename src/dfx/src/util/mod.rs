use crate::lib::error::{DfxError, DfxResult};
use candid::parser::typing::{check_prog, ActorEnv, TypeEnv};
use candid::types::Function;
use candid::{IDLArgs, IDLProg};
use ic_agent::Blob;

pub mod assets;
pub mod clap;

/// Deserialize and print return values from canister method.
pub fn print_idl_blob(blob: &Blob) -> Result<(), candid::Error> {
    let result = candid::IDLArgs::from_bytes(&(*blob.0));
    if result.is_err() {
        let hex_string = hex::encode(&(*blob.0));
        eprintln!("Error deserializing blob 0x{}", hex_string);
    }
    println!("{}", result?);
    Ok(())
}

/// Parse IDL file into TypeEnv. This is a best effort function: it will succeed if
/// the IDL file can be parsed and type checked in Rust parser, and has an
/// actor in the IDL file. If anything fails, it returns None.
pub fn check_candid_file(idl_path: &std::path::Path) -> Option<(TypeEnv, ActorEnv)> {
    let idl_file = std::fs::read_to_string(idl_path).ok()?;
    let ast = idl_file.parse::<IDLProg>().ok()?;
    let mut env = TypeEnv::new();
    let actor = check_prog(&mut env, &ast).ok()?;
    if actor.is_empty() {
        None
    } else {
        Some((env, actor))
    }
}

pub fn blob_from_arguments(
    arguments: Option<&str>,
    arg_type: Option<&str>,
    method_type: Option<(TypeEnv, Function)>,
) -> DfxResult<Blob> {
    let arg_type = arg_type.unwrap_or("idl");
    match arg_type {
        "raw" => {
            let bytes = hex::decode(&arguments.unwrap_or("")).map_err(|e| {
                DfxError::InvalidArgument(format!("Argument is not a valid hex string: {}", e))
            })?;
            Ok(bytes)
        }
        "idl" => {
            let arguments = arguments.unwrap_or("()");
            let args: IDLArgs = arguments
                .parse()
                .map_err(|e| DfxError::InvalidArgument(format!("Invalid IDL: {}", e)))?;
            let typed_args = match method_type {
                None => {
                    eprintln!("cannot find method type, dfx will send message with inferred type");
                    args.to_bytes()
                }
                Some((env, func)) => args.to_bytes_with_types(&env, &func.args),
            }
            .map_err(|e| DfxError::InvalidData(format!("Unable to convert IDL to bytes: {}", e)))?;
            Ok(typed_args)
        }
        v => Err(DfxError::Unknown(format!("Invalid type: {}", v))),
    }
    .map(Blob::from)
}
