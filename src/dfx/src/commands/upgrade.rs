use crate::lib::dfxvm::dfxvm_released;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::manifest::{get_latest_release, get_latest_version};
use anyhow::{bail, Context};
use clap::Parser;
use semver::Version;

/// Upgrade DFX.
#[derive(Parser)]
pub struct UpgradeOpts {
    /// Current Version.
    #[arg(long)]
    current_version: Option<String>,

    #[arg(long, default_value = "https://sdk.dfinity.org", hide = true)]
    release_root: String,
}

pub fn exec(env: &dyn Environment, opts: UpgradeOpts) -> DfxResult {
    if dfxvm_released()? {
        println!("dfx upgrade has been disabled. Please use the dfx version manager (dfxvm) to upgrade.");
        println!("You can install dfxvm by running the following command:");
        println!();
        println!(r#"    sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)""#);
        println!();
        bail!("dfx upgrade is disabled");
    }

    // Find OS architecture.
    let os_arch = match std::env::consts::OS {
        "linux" => "x86_64-linux",
        "macos" => "x86_64-darwin",
        _ => panic!("Not supported architecture"),
    };
    let current_version = if let Some(version) = opts.current_version {
        Version::parse(&version)
            .with_context(|| format!("Failed to parse {} as version.", &version))?
    } else {
        env.get_version().clone()
    };

    println!("Current version: {}", current_version);
    let release_root = opts.release_root.as_str();
    let latest_version = get_latest_version(release_root, None)?;

    if latest_version > current_version {
        println!("New version available: {}", latest_version);
        get_latest_release(release_root, &latest_version, os_arch)?;
    } else {
        println!("Already up to date");
    }

    Ok(())
}
