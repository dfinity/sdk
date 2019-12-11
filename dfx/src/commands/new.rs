use super::upgrade::{get_latest_version, is_upgrade_necessary};
use crate::config::dfinity::CONFIG_FILE_NAME;
use crate::lib::env::{BinaryCacheEnv, PlatformEnv, VersionEnv};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::assets;
use clap::{App, Arg, ArgMatches, SubCommand};
use console::{style, Style};
use indicatif::HumanBytes;
use indicatif::{ProgressBar, ProgressDrawTarget};
use lazy_static::lazy_static;
use semver::Version;
use serde_json::Value;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tar::Archive;

const DRY_RUN: &str = "dry_run";
const PROJECT_NAME: &str = "project_name";
const RELEASE_ROOT: &str = "https://sdk.dfinity.org";
lazy_static! {
// Tested on a phone tethering connection. This should be fine with
// little impact to the user, given that "new" is supposedly a
// heavy-weight operation. Thus, worst case we are utilizing the user
// expectation for the duration to have a more expensive version
// check.
    static ref CHECK_VERSION_TIMEOUT: Duration = Duration::from_secs(2);
}

/// Validate a String can be a valid project name.
/// A project name is valid if it starts with a letter, and is alphanumeric (with hyphens).
/// It cannot end with a dash.
pub fn project_name_validator(name: String) -> Result<(), String> {
    let mut chars = name.chars();
    // Check first character first. If there's no first character it's empty.
    if let Some(first) = chars.next() {
        if first.is_ascii_alphabetic() {
            // Then check all other characters.
            // Reverses the search here; if there is a character that is not compatible
            // it is found and an error is returned.
            let m: Vec<&str> = name
                .matches(|x: char| !x.is_ascii_alphanumeric() && x != '_')
                .collect();

            if m.is_empty() {
                Ok(())
            } else {
                Err(format!(
                    r#"Invalid character(s): "{}""#,
                    m.iter()
                        .fold(String::new(), |acc, &num| acc + &num.to_string())
                ))
            }
        } else {
            Err("Must start with a letter.".to_owned())
        }
    } else {
        Err("Cannot be empty.".to_owned())
    }
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("new")
        .about(UserMessage::CreateProject.to_str())
        .arg(
            Arg::with_name(PROJECT_NAME)
                .help(UserMessage::ProjectName.to_str())
                .validator(project_name_validator)
                .required(true),
        )
        .arg(
            Arg::with_name(DRY_RUN)
                .help(UserMessage::DryRun.to_str())
                .long("dry-run")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("frontend")
                .help(UserMessage::NewFrontend.to_str())
                .takes_value(false),
        )
}

enum Status<'a> {
    Create(&'a Path, usize),
    CreateDir(&'a Path),
}

impl std::fmt::Display for Status<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Status::Create(path, size) => write!(
                f,
                "{:<12} {} ({})...",
                style("CREATE").green().bold(),
                path.to_str().unwrap_or("<unknown>"),
                HumanBytes(*size as u64),
            )?,
            Status::CreateDir(path) => write!(
                f,
                "{:<12} {}...",
                style("CREATE_DIR").blue().bold(),
                path.to_str().unwrap_or("<unknown>"),
            )?,
        };

        Ok(())
    }
}

pub fn create_file(path: &Path, content: &str, dry_run: bool) -> DfxResult {
    if !dry_run {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(&path, content)?;
    }

    eprintln!("{}", Status::Create(path, content.len()));
    Ok(())
}

#[allow(dead_code)]
pub fn create_dir<P: AsRef<Path>>(path: P, dry_run: bool) -> DfxResult {
    let path = path.as_ref();
    if path.is_dir() {
        return Ok(());
    }

    if !dry_run {
        std::fs::create_dir_all(&path)?;
    }

    eprintln!("{}", Status::CreateDir(path));
    Ok(())
}

pub fn init_git(project_name: &Path) -> DfxResult {
    let init_status = std::process::Command::new("git")
        .arg("init")
        .current_dir(project_name)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status();

    if init_status.is_ok() && init_status.unwrap().success() {
        eprintln!("Creating git repository...");
        std::process::Command::new("git")
            .arg("add")
            .current_dir(project_name)
            .arg(".")
            .output()?;
        std::process::Command::new("git")
            .arg("commit")
            .current_dir(project_name)
            .arg("-a")
            .arg("--message=Initial commit.")
            .output()?;
    }

    Ok(())
}

fn write_files_from_entries<R: Sized + Read>(
    archive: &mut Archive<R>,
    root: &Path,
    dry_run: bool,
    variables: &HashMap<String, String>,
) -> DfxResult {
    for entry in archive.entries()? {
        let mut file = entry?;

        if file.header().entry_type().is_dir() {
            continue;
        }

        let mut s = String::new();
        file.read_to_string(&mut s)
            .or_else(|e| Err(DfxError::Io(e)))?;

        // Perform replacements.
        variables.iter().for_each(|(name, value)| {
            let pattern = "{".to_owned() + name + "}";
            s = s.replace(pattern.as_str(), value);
        });

        // Perform path replacements.
        let mut p = root
            .join(file.header().path()?)
            .to_str()
            .expect("Non unicode project name path.")
            .to_string();

        variables.iter().for_each(|(name, value)| {
            let pattern = "__".to_owned() + name + "__";
            p = p.replace(pattern.as_str(), value);
        });

        let p = PathBuf::from(p);
        create_file(p.as_path(), s.as_str(), dry_run)?;
    }

    Ok(())
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryCacheEnv + PlatformEnv + VersionEnv,
{
    let dry_run = args.is_present(DRY_RUN);
    let project_name_path = args
        .value_of(PROJECT_NAME)
        .ok_or_else(|| DfxError::InvalidArgument("project_path".to_string()))?;
    let project_name = Path::new(project_name_path);

    if project_name.exists() {
        return Err(DfxError::ProjectExists);
    }

    let dfx_version = env.get_version();

    let current_version = Version::parse(dfx_version)
        .map_err(|e| DfxError::InvalidData(format!("invalid version: {}", e)))?;

    // It is fine for the following command to timeout or fail. We
    // drop the error.
    let latest_version = get_latest_version(RELEASE_ROOT, Some(*CHECK_VERSION_TIMEOUT)).ok();

    if is_upgrade_necessary(latest_version.clone(), current_version.clone()) {
        warn_upgrade(latest_version, current_version);
    }

    eprintln!(r#"Creating new project "{}"..."#, project_name.display());
    if dry_run {
        eprintln!(r#"Running in dry mode. Nothing will be committed to disk."#);
    }

    let project_name_str = project_name
        .to_str()
        .ok_or_else(|| DfxError::InvalidArgument("project_name".to_string()))?;

    let variables: HashMap<String, String> = [
        ("project_name".to_string(), project_name_str.to_string()),
        ("dfx_version".to_string(), dfx_version.to_string()),
        ("dot".to_string(), ".".to_string()),
    ]
    .iter()
    .cloned()
    .collect();

    let mut new_project_files = assets::new_project_files()?;
    write_files_from_entries(&mut new_project_files, project_name, dry_run, &variables)?;

    let node_installed = std::process::Command::new("node")
        .arg("--version")
        .output()
        .is_ok();

    // Only update the dfx.json if we're not running in dry run.
    if !dry_run {
        if node_installed || args.is_present("frontend") {
            // Check if node is available, and if it is create the files for the frontend build.
            let mut new_project_node_files = assets::new_project_node_files()?;
            write_files_from_entries(
                &mut new_project_node_files,
                project_name,
                dry_run,
                &variables,
            )?;

            let dfx_path = project_name.join(CONFIG_FILE_NAME);
            let content = std::fs::read(&dfx_path)?;
            let mut config_json: Value =
                serde_json::from_slice(&content).map_err(std::io::Error::from)?;

            let frontend_value: serde_json::Map<String, Value> = [(
                "entrypoint".to_string(),
                ("src/".to_owned() + project_name_str + "/public/index.js").into(),
            )]
            .iter()
            .cloned()
            .collect();

            let p = config_json
                .pointer_mut(("/canisters/".to_owned() + project_name_str).as_str())
                .unwrap();
            p.as_object_mut()
                .unwrap()
                .insert("frontend".to_string(), Value::from(frontend_value));
            let pretty = serde_json::to_string_pretty(&config_json).or_else(|e| {
                Err(DfxError::InvalidData(format!(
                    "Failed to serialize dfx.json: {}",
                    e
                )))
            })?;
            std::fs::write(&dfx_path, pretty)?;

            let b = ProgressBar::new_spinner();
            b.set_draw_target(ProgressDrawTarget::stderr());

            b.set_message("Installing node dependencies...");
            b.enable_steady_tick(80);

            // Install node modules. Error is not blocking, we just show a message instead.
            if std::process::Command::new("npm")
                .arg("install")
                .arg("--quiet")
                .arg("--no-progress")
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .current_dir(project_name)
                .status()
                .is_ok()
            {
                b.finish_with_message("Done.");
            } else {
                b.finish_with_message("An error occured. See the messages above for more details.");
            }
        } else if !args.is_present("frontend") && !node_installed {
            eprintln!("Node could not be found. Skipping installing the frontend example code.");
            eprintln!("\nYou can bypass this check by using the --frontend flag.")
        }
    }

    if !dry_run {
        // If on mac, we should validate that XCode toolchain was installed.
        #[cfg(target_os = "macos")]
        {
            let mut should_git = true;
            if let Ok(code) = std::process::Command::new("xcode-select")
                .arg("-p")
                .stderr(Stdio::null())
                .stdout(Stdio::null())
                .status()
            {
                if !code.success() {
                    // git is not installed.
                    should_git = false;
                }
            } else {
                // Could not find XCode Toolchain on Mac, that's weird.
                should_git = false;
            }

            if should_git {
                init_git(&project_name)?;
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            init_git(&project_name)?;
        }
    }

    // Print welcome message.
    eprintln!(
        // This needs to be included here because we cannot use the result of a function for
        // the format!() rule (and so it cannot be moved in the util::assets module).
        include_str!("../../assets/welcome.txt"),
        dfx_version,
        assets::dfinity_logo(),
        project_name_str
    );

    Ok(())
}

fn warn_upgrade(latest_version: Option<Version>, current_version: Version) {
    eprintln!("You seem to be running an outdated version of dfx.");

    let red = Style::new().red();
    let green = Style::new().green();
    let yellow = Style::new().yellow();

    eprint!(
        "\nCurrent version: {}",
        red.apply_to(current_version.clone())
    );
    if let Some(v) = latest_version {
        eprint!("{}", yellow.apply_to("→"));
        eprintln!(" latest version: {}", green.apply_to(v));
    }
    eprintln!("\nYou are strongly encouraged to upgrade by running 'dfx upgrade'!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_name_is_valid() {
        assert!(project_name_validator("a".to_owned()).is_ok());
        assert!(project_name_validator("a_".to_owned()).is_ok());
        assert!(project_name_validator("a_1".to_owned()).is_ok());
        assert!(project_name_validator("A".to_owned()).is_ok());
        assert!(project_name_validator("A1".to_owned()).is_ok());
        assert!(project_name_validator("a_good_name_".to_owned()).is_ok());
        assert!(project_name_validator("a_good_name".to_owned()).is_ok());
    }

    #[test]
    fn project_name_is_invalid() {
        assert!(project_name_validator("_a_good_name_".to_owned()).is_err());
        assert!(project_name_validator("__also_good".to_owned()).is_err());
        assert!(project_name_validator("_1".to_owned()).is_err());
        assert!(project_name_validator("_a".to_owned()).is_err());
        assert!(project_name_validator("1".to_owned()).is_err());
        assert!(project_name_validator("1_".to_owned()).is_err());
        assert!(project_name_validator("-".to_owned()).is_err());
        assert!(project_name_validator("_".to_owned()).is_err());
        assert!(project_name_validator("a-b-c".to_owned()).is_err());
        assert!(project_name_validator("🕹".to_owned()).is_err());
        assert!(project_name_validator("不好".to_owned()).is_err());
        assert!(project_name_validator("a:b".to_owned()).is_err());
    }
}
