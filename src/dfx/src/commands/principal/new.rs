#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::{blob_from_arguments, print_idl_blob};
use clap::{App, Arg, ArgMatches, SubCommand};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("new")
        .about(UserMessage::QueryCanister.to_str())
        .arg(
            Arg::with_name("with")
                .help(UserMessage::PrincipalNew.to_str())
                .long("type"),
        )
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    let _config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    Ok(())
}
