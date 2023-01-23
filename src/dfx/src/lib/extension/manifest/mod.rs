//! directory contains code that parses the .json files

pub mod compatibility_matrix;
pub mod extension;

/// `compatibility.json` is a file describing the compatibility
/// matrix between extensions versions and the dfx version
pub use compatibility_matrix::ExtensionsCompatibilityMatrix;
/// `manifest.json` is a file describing the extension
pub use extension::ExtensionManifest;
