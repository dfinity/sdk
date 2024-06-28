//! Directory contains code that parses the .json files.

pub mod compatibility_matrix;
pub mod dependencies;
pub mod extension;

/// `compatibility.json` is a file describing the compatibility
/// matrix between extensions versions and the dfx version.
pub use compatibility_matrix::ExtensionCompatibilityMatrix;

/// A file that lists the dependencies of all versions of an extension.
pub use dependencies::ExtensionDependencies;

/// A file that describes an extension.
pub use extension::ExtensionManifest;
