// directory contains code that parsers the .json files
// - compatibility_matrix is a top level file desriging
//   the compatibility matrix between extensions versions
//   and the dfx version
// - extension_spac is a file describing the extension

pub mod compatibility_matrix;
pub mod extension;

pub use compatibility_matrix::ExtensionsCompatibilityMatrix;
pub use extension::ExtensionManifest;
