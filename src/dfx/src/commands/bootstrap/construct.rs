//! File       : construct.rs
//! License    : Apache 2.0 with LLVM Exceptions
//! Copyright  : 2020 DFINITY Stiftung
//! Maintainer : Enzo Haussecker <enzo@dfinity.org>
//! Stability  : Experimental

use crate::lib::message::UserMessage;
use clap::{App, Arg, SubCommand};

/// TODO (enzo): Documentation.
pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("bootstrap")
        .about(UserMessage::BootstrapCommand.to_str())
        .arg(
            Arg::with_name("ip")
                .help(UserMessage::BootstrapIP.to_str())
                .long("ip")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .help(UserMessage::BootstrapPort.to_str())
                .long("port")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("providers")
                .help(UserMessage::BootstrapProviders.to_str())
                .long("providers")
                .multiple(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("root")
                .help(UserMessage::BootstrapRoot.to_str())
                .long("root")
                .takes_value(true),
        )
}
