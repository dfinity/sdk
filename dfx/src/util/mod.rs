use ic_http_agent::Blob;

pub mod assets;
pub mod clap;

pub fn print_idl_blob(blob: &Blob) -> Result<(), serde_idl::Error> {
    let result = serde_idl::decode_value(&(*blob.0))?;
    println!("{}", result);
    Ok(())
}
