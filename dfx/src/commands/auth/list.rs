use crate::lib::error::DfxResult;
use clap::{App, ArgMatches, SubCommand};

pub fn available() -> bool {
    true
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("list")
        .about("List all authentications for this user.")
}

pub fn exec(_args: &ArgMatches<'_>) -> DfxResult {
    let mut v = Vec::new();

    if let Ok(username) = std::env::var("USER") {
        v.push(username);
    }
    v.push("fake_user_1".to_owned());


    for username in v {
        println!("{}", username);
    }

    Ok(())
}
