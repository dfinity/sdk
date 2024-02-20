use std::path::PathBuf;

use clap::Args;

use crate::lib::error::DfxResult;
use crate::util::arguments_from_file;
use crate::util::clap::parsers::file_or_stdin_parser;

#[derive(Args, Clone, Debug, Default)]
pub struct ArgumentFromCliOpt {
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

impl ArgumentFromCliOpt {
    pub fn get_argument(&self) -> DfxResult<(Option<String>, Option<String>)> {
        get_argument_from_cli(&self.argument, &self.argument_type, &self.argument_file)
    }
}

pub fn get_argument_from_cli(
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
