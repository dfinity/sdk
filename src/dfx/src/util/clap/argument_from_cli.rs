//! This module contains the CLI options for specifying an argument to pass to a method.
//!
//! # Notice
//! There are two variants: `ArgumentFromCliOpt1` and `ArgumentFromCliOpt2`.
//!
//! Opt1 is used in:
//! - `dfx deploy`
//! - `dfx canister install`
//! - `dfx deps init`
//!
//! Opt2 is used in:
//! - `dfx canister call`
//! - `dfx canister sign`
//!
//! They are different in two points:
//! - `argument`
//!   - In Opt1, it is a "long" option, it must be set with `--argument <ARGUMENT>` or `--argument=<ARGUMENT>`.
//!   - In Opt2, it is a "positional" option, e.g. `dfx canister call <CANISTER_NAME> <METHOD_NAME> [ARGUMENT]`
//! - name of the field for the argument type
//!   - In Opt1, it is `argument_type`.
//!   - In Opt2, it is `type`.
use std::path::PathBuf;

use clap::Args;

use crate::lib::error::DfxResult;
use crate::util::arguments_from_file;
use crate::util::clap::parsers::file_or_stdin_parser;

/// CLI options for specifying an argument to pass to a method (Variant 1).
///
/// Check the module level documentation for more details.
#[derive(Args, Clone, Debug, Default)]
pub struct ArgumentFromCliOpt1 {
    /// Specifies the argument to pass to the method.
    #[arg(long, conflicts_with("argument_file"))]
    argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[arg(long, requires("argument"), value_parser = ["idl", "raw"])]
    argument_type: Option<String>,

    /// Specifies the file from which to read the argument to pass to the method.
    #[arg(long, value_parser = file_or_stdin_parser, conflicts_with("argument"))]
    argument_file: Option<PathBuf>,
}

impl ArgumentFromCliOpt1 {
    pub fn get_argument_and_type(&self) -> DfxResult<(Option<String>, Option<String>)> {
        get_argument_from_cli(&self.argument, &self.argument_type, &self.argument_file)
    }
}

/// CLI options for specifying an argument to pass to a method (Variant 2).
///
/// Check the module level documentation for more details.
#[derive(Args, Clone, Debug, Default)]
pub struct ArgumentFromCliOpt2 {
    /// Specifies the argument to pass to the method.
    #[arg(conflicts_with("argument_file"))]
    argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[arg(long, requires("argument"), value_parser = ["idl", "raw"])]
    r#type: Option<String>,

    /// Specifies the file from which to read the argument to pass to the method.
    #[arg(long, value_parser = file_or_stdin_parser, conflicts_with("argument"))]
    argument_file: Option<PathBuf>,
}

impl ArgumentFromCliOpt2 {
    pub fn get_argument_and_type(&self) -> DfxResult<(Option<String>, Option<String>)> {
        get_argument_from_cli(&self.argument, &self.r#type, &self.argument_file)
    }
}

fn get_argument_from_cli(
    argument: &Option<String>,
    argument_type: &Option<String>,
    argument_file: &Option<PathBuf>,
) -> DfxResult<(Option<String>, Option<String>)> {
    let arguments_from_file = argument_file
        .as_deref()
        .map(arguments_from_file)
        .transpose()?;
    let arguments = argument.clone();
    let argument_from_cli = arguments_from_file.or(arguments);
    Ok((argument_from_cli, argument_type.clone()))
}
