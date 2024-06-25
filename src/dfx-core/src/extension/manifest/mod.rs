//! Directory contains code that parses the .json files.

pub mod compatibility_matrix;
/// `compatibility.json` is a file describing the compatibility
/// matrix between extensions versions and the dfx version.
pub use compatibility_matrix::ExtensionCompatibilityMatrix;
/// URL to the `compatibility.json` file.
pub use compatibility_matrix::COMMON_EXTENSIONS_MANIFEST_LOCATION;

pub mod extension;
pub mod dependencies;

/// `manifest.json` is a file describing the extension.
pub use extension::ExtensionManifest;
/// File name for the file describing the extension.
pub use extension::MANIFEST_FILE_NAME;
