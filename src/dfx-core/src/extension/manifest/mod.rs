//! Directory contains code that parses the .json files.

pub mod dependencies;
pub mod extension;

/// A file that lists the dependencies of all versions of an extension.
pub use dependencies::ExtensionDependencies;

/// A file that describes an extension.
pub use extension::ExtensionManifest;
