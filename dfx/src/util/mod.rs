use ic_http_agent::Blob;
use serde::Deserialize;

pub mod assets;
pub mod clap;

/// Deserialize an IDL Stream into T, or fail if the stream is not the right type.
/// This function is used to try to guess the IDL format.
fn maybe_get<'de, T>(blob: &'de Blob) -> Result<T, serde_idl::Error>
where
    T: Deserialize<'de>,
{
    let mut de = serde_idl::de::IDLDeserialize::new(&(*blob.0));
    let t: T = de.get_value::<T>()?;
    de.done()?;

    Ok(t)
}

/// Try to deserialize and print out the type T, or return an error.
fn maybe_write<'de, T>(blob: &'de Blob) -> Result<(), serde_idl::Error>
where
    T: Deserialize<'de> + std::fmt::Display,
{
    let t: T = maybe_get(blob)?;
    println!("{}", t);
    Ok(())
}

/// Try to print the IDL blob by figuring out its type through trial & error.
/// This should be superseded by a proper IDL ASCII format that is human readable,
/// but until then we try to get an integer or a string out of it.
pub fn print_idl_blob(blob: &Blob) -> Result<(), serde_idl::Error> {
    // Try unit, vector of unit, u64, or else string, else return an error.
    // Use `maybe_get` directly as the Unit type does not implement the Display trait.
    if maybe_get::<()>(blob).is_ok()
        || maybe_get::<Vec<()>>(blob).is_ok()
        || maybe_write::<u64>(blob).is_ok()
        || maybe_write::<i64>(blob).is_ok()
        || maybe_write::<bool>(blob).is_ok()
    {
        Ok(())
    } else {
        maybe_write::<String>(blob)
    }
}
