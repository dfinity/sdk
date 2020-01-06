use crate::lib::environment::Environment;
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
    println!("{}", result?);
    Ok(())
}

/// Parse IDL file into AST. This is a best effort function: it will succeed if
/// the IDL file can be type checked by didc, parsed in Rust parser, and has an
/// actor in the IDL file. If anything fails, it returns None.
pub fn load_idl_file(env: &dyn Environment, idl_path: &std::path::Path) -> Option<IDLProg> {
    let mut didc = env.get_cache().get_binary_command("didc").ok()?;
    let status = didc.arg("--check").arg(&idl_path).status().ok()?;
    if !status.success() {
        return None;
    }
    let idl_file = std::fs::read_to_string(idl_path).ok()?;
    let ast = idl_file.parse::<IDLProg>().ok()?;
    if ast.actor.is_some() {
        Some(ast)
    } else {
        None
    }
}
