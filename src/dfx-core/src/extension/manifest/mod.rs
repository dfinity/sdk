//! Directory contains code that parses the .json files.

pub mod compatibility_matrix;
/// `compatibility.json` is a file describing the compatibility
/// matrix between extensions versions and the dfx version.
pub use compatibility_matrix::ExtensionCompatibilityMatrix;
/// URL to the `compatibility.json` file.
pub use compatibility_matrix::COMMON_EXTENSIONS_MANIFEST_LOCATION;

pub mod extension;
/// `manifest.json` is a file describing the extension.
pub use extension::ExtensionManifest;
/// File name for the file describing the extension.
pub use extension::MANIFEST_FILE_NAME;

pub mod external_extension;
/// In order for extensions stored in external repositories to be consumable by dfx,
/// the repository maintainers must provide a URL to a JSON file with the following structure.
/// This file is an amalgamation of the compatibility.json file and the combined manifest.json
/// files for each individual extension, as they exist in the DFINITY extension repository.
/// The file must respect the following constraint: for every version of every extension
/// listed under the “compatibility” key, there should be a corresponding entry under the “extensions” key.
pub use external_extension::ExternalExtensionManifest;
