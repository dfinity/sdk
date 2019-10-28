use ic_http_agent::Blob;

pub mod assets;
pub mod clap;

pub fn print_idl_blob(blob: &Blob) -> Result<(), serde_idl::Error> {
    let result = serde_idl::IDLArgs::from_bytes(&(*blob.0));
    if result.is_err() {
        let hex_string = hex::encode(&(*blob.0));
        println!("Error deserializing blob {}", hex_string);
    }
    println!("{}", result?);
    Ok(())
}
