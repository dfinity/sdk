use crate::lib::env::BinaryResolverEnv;
use ic_http_agent::Blob;
use serde_idl::IDLProg;

pub mod assets;
pub mod clap;

/// Deserialize and print return values from canister method.
pub fn print_idl_blob(blob: &Blob) -> Result<(), serde_idl::Error> {
    let result = serde_idl::IDLArgs::from_bytes(&(*blob.0));
    if result.is_err() {
        let hex_string = hex::encode(&(*blob.0));
        eprintln!("Error deserializing blob 0x{}", hex_string);
    }
    let result = result?;
    if result.args.len() == 1 {
        println!("{}", result.args[0]);
    } else {
        println!("{}", result);
    }
    Ok(())
}

/// Parse IDL file into AST. This is a best effort function: it will succeed if
/// the IDL file can be type checked by didc, parsed in Rust parser, and has an
/// actor in the IDL file. If anything fails, it returns None.
pub fn load_idl_file<T>(env: &T, idl_path: &std::path::Path) -> Option<IDLProg>
where
    T: BinaryResolverEnv,
{
    let didc = env.get_binary_command("didc");
    if didc.is_err() {
        return None;
    }
    let valid_idl = didc.unwrap().arg("--check").arg(&idl_path).status();
    if valid_idl.is_err() || !valid_idl.unwrap().success() {
        return None;
    }
    let idl_file = std::fs::read_to_string(idl_path);
    if idl_file.is_err() {
        return None;
    }
    match idl_file.unwrap().parse::<IDLProg>() {
        Ok(ast) => {
            if ast.actor.is_some() {
                Some(ast)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}
