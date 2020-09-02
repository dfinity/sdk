use crate::lib::error::{DfxError, DfxResult};
use candid::parser::typing::{check_prog, TypeEnv};
use candid::types::{Function, Type};
use candid::{parser::value::IDLValue, IDLArgs, IDLProg};

pub mod assets;
pub mod clap;

/// Deserialize and print return values from canister method.
pub fn print_idl_blob(
    blob: &[u8],
    output_type: Option<&str>,
    method_type: &Option<(TypeEnv, Function)>,
) -> DfxResult<()> {
    let output_type = output_type.unwrap_or("idl");
    match output_type {
        "raw" => {
            let hex_string = hex::encode(blob);
            println!("{}", hex_string);
        }
        "idl" => {
            let result = match method_type {
                None => candid::IDLArgs::from_bytes(blob),
                Some((env, func)) => candid::IDLArgs::from_bytes_with_types(blob, &env, &func.rets),
            };
            if result.is_err() {
                let hex_string = hex::encode(blob);
                eprintln!("Error deserializing blob 0x{}", hex_string);
            }
            println!("{}", result?);
        }
        v => return Err(DfxError::Unknown(format!("Invalid output type: {}", v))),
    }
    Ok(())
}

/// Parse IDL file into TypeEnv. This is a best effort function: it will succeed if
/// the IDL file can be parsed and type checked in Rust parser, and has an
/// actor in the IDL file. If anything fails, it returns None.
pub fn get_candid_type(
    idl_path: &std::path::Path,
    method_name: &str,
) -> Option<(TypeEnv, Function)> {
    let (env, ty) = check_candid_file(idl_path).ok()?;
    let actor = ty?;
    let method = env.get_method(&actor, method_name).ok()?.clone();
    Some((env, method))
}

pub fn check_candid_file(idl_path: &std::path::Path) -> DfxResult<(TypeEnv, Option<Type>)> {
    let idl_file = std::fs::read_to_string(idl_path)?;
    let ast = idl_file.parse::<IDLProg>()?;
    let mut env = TypeEnv::new();
    let actor = check_prog(&mut env, &ast)?;
    Ok((env, actor))
}

pub fn blob_from_arguments(
    arguments: Option<&str>,
    arg_type: Option<&str>,
    method_type: &Option<(TypeEnv, Function)>,
) -> DfxResult<Vec<u8>> {
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
            let args: DfxResult<IDLArgs> =
                arguments.parse::<IDLArgs>().map_err(|e: candid::Error| {
                    DfxError::InvalidArgument(format!("Invalid Candid values: {}", e))
                });
            let typed_args = match method_type {
                None => {
                    eprintln!("cannot find method type, dfx will send message with inferred type");
                    args?.to_bytes()
                }
                Some((env, func)) => {
                    let first_char = arguments.chars().next();
                    let is_candid_format = first_char.map_or(false, |c| c == '(');
                    // If parsing fails and method expects a single value, try parsing as IDLValue.
                    // If it still fails, and method expects a text type, send arguments as text.
                    let args = args.or_else(|e| {
                        if func.args.len() == 1 && !is_candid_format {
                            let is_quote = first_char.map_or(false, |c| c == '"');
                            if candid::types::Type::Text == func.args[0] && !is_quote {
                                Ok(IDLValue::Text(arguments.to_string()))
                            } else {
                                arguments.parse::<IDLValue>().map_err(|e| {
                                    DfxError::InvalidArgument(format!(
                                        "Invalid Candid values: {}",
                                        e
                                    ))
                                })
                            }
                            .map(|v| IDLArgs::new(&[v]))
                        } else {
                            Err(e)
                        }
                    });
                    args?.to_bytes_with_types(&env, &func.args)
                }
            }
            .map_err(|e| {
                DfxError::InvalidData(format!("Unable to serialize Candid values: {}", e))
            })?;
            Ok(typed_args)
        }
        v => Err(DfxError::Unknown(format!("Invalid type: {}", v))),
    }
}
