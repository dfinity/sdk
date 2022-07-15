use crate::config::dfinity::CONFIG_FILE_NAME;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::manifest::{get_latest_version, is_upgrade_necessary};
use crate::util::assets;
use crate::util::clap::validators::project_name_validator;

use anyhow::{anyhow, bail, Context};
use clap::Parser;
use console::{style, Style};
use fn_error_context::context;
use indicatif::HumanBytes;
use lazy_static::lazy_static;
use semver::Version;
use serde_json::Value;
use slog::{info, warn, Logger};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tar::Archive;

// const DRY_RUN: &str = "dry_run";
// const PROJECT_NAME: &str = "project_name";
const RELEASE_ROOT: &str = "https://sdk.dfinity.org";

// The dist-tag to use when getting the version from NPM.
const AGENT_JS_DEFAULT_INSTALL_DIST_TAG: &str = "latest";

lazy_static! {
// Tested on a phone tethering connection. This should be fine with
// little impact to the user, given that "new" is supposedly a
// heavy-weight operation. Thus, worst case we are utilizing the user
// expectation for the duration to have a more expensive version
// check.
    static ref CHECK_VERSION_TIMEOUT: Duration = Duration::from_secs(2);

}

/// Creates a new project.
#[derive(Parser)]
pub struct NewOpts {
    /// Specifies the name of the project to create.
    #[clap(validator(project_name_validator))]
    project_name: String,

    /// Choose the type of canister in the starter project. Default to be motoko.
    #[clap(long, possible_values(&["motoko", "rust"]), default_value = "motoko")]
    r#type: String,

    /// Provides a preview the directories and files to be created without adding them to the file system.
    #[clap(long)]
    dry_run: bool,

    /// Installs the frontend code example for the default canister. This defaults to true if Node is installed, or false if it isn't.
    #[clap(long)]
    frontend: bool,

    #[clap(long, conflicts_with = "frontend")]
    no_frontend: bool,

    /// Overrides which version of the JavaScript Agent to install. By default, will contact
    /// NPM to decide.
    #[clap(long, requires("frontend"))]
    agent_version: Option<String>,
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

pub fn create_file(log: &Logger, path: &Path, content: &[u8], dry_run: bool) -> DfxResult {
    if !dry_run {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)
                .with_context(|| format!("Failed to create directory {}.", p.to_string_lossy()))?;
        }
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write to {}.", path.to_string_lossy()))?;
    }

    info!(log, "{}", Status::Create(path, content.len()));
    Ok(())
}

#[allow(dead_code)]
pub fn create_dir<P: AsRef<Path>>(log: &Logger, path: P, dry_run: bool) -> DfxResult {
    let path = path.as_ref();
    if path.is_dir() {
        return Ok(());
    }

    if !dry_run {
        std::fs::create_dir_all(&path)
            .with_context(|| format!("Failed to create directory {}.", path.to_string_lossy()))?;
    }

    info!(log, "{}", Status::CreateDir(path));
    Ok(())
}

#[context("Failed to init git at {}.", project_name.to_string_lossy())]
pub fn init_git(log: &Logger, project_name: &Path) -> DfxResult {
    let init_status = std::process::Command::new("git")
        .arg("init")
        .current_dir(project_name)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .status();

    if init_status.is_ok() && init_status.unwrap().success() {
        info!(log, "Creating git repository...");
        std::process::Command::new("git")
            .arg("add")
            .current_dir(project_name)
            .arg(".")
            .output()
            .context("Failed to run 'git add'.")?;
        std::process::Command::new("git")
            .arg("commit")
            .current_dir(project_name)
            .arg("-a")
            .arg("--message=Initial commit.")
            .output()
            .context("Failed to run 'git commit'.")?;
    }

    Ok(())
}

#[context("Failed to unpack archive to {}.", root.to_string_lossy())]
fn write_files_from_entries<R: Sized + Read>(
    log: &Logger,
    archive: &mut Archive<R>,
    root: &Path,
    dry_run: bool,
    variables: &BTreeMap<String, String>,
) -> DfxResult {
    for entry in archive.entries()? {
        let mut file = entry?;

        if file.header().entry_type().is_dir() {
            continue;
        }

        let mut v = Vec::new();
        file.read_to_end(&mut v).map_err(DfxError::from)?;

        let v = match String::from_utf8(v) {
            Err(err) => err.into_bytes(),
            Ok(mut s) => {
                // Perform replacements.
                variables.iter().for_each(|(name, value)| {
                    let pattern = "{".to_owned() + name + "}";
                    s = s.replace(pattern.as_str(), value);
                });
                s.into_bytes()
            }
        };

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
        create_file(log, p.as_path(), &v, dry_run)?;
    }

    Ok(())
}

#[context("Failed to run 'npm install'.")]
fn npm_install(location: &Path) -> DfxResult<std::process::Child> {
    std::process::Command::new("npm")
        .arg("install")
        .arg("--quiet")
        .arg("--no-progress")
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .current_dir(location)
        .spawn()
        .map_err(DfxError::from)
}

#[context("Failed to scaffold frontend code.")]
fn scaffold_frontend_code(
    env: &dyn Environment,
    dry_run: bool,
    project_name: &Path,
    arg_no_frontend: bool,
    arg_frontend: bool,
    agent_version: &Option<String>,
    variables: &BTreeMap<String, String>,
) -> DfxResult {
    let log = env.get_logger();
    let node_installed = std::process::Command::new("node")
        .arg("--version")
        .output()
        .is_ok();

    let project_name_str = project_name
        .to_str()
        .ok_or_else(|| anyhow!("Invalid argument: project_name"))?;

    if (node_installed && !arg_no_frontend) || arg_frontend {
        // Check if node is available, and if it is create the files for the frontend build.
        let js_agent_version = if let Some(v) = agent_version {
            v.clone()
        } else {
            get_agent_js_version_from_npm(AGENT_JS_DEFAULT_INSTALL_DIST_TAG)
                .map_err(|err| anyhow!("Cannot execute npm: {}", err))?
        };

        let mut variables = variables.clone();
        variables.insert("js_agent_version".to_string(), js_agent_version);
        variables.insert(
            "project_name_uppercase".to_string(),
            project_name_str.to_uppercase(),
        );

        let mut new_project_node_files = assets::new_project_node_files()?;
        write_files_from_entries(
            log,
            &mut new_project_node_files,
            project_name,
            dry_run,
            &variables,
        )?;

        let dfx_path = project_name.join(CONFIG_FILE_NAME);
        let content =
            std::fs::read(&dfx_path).with_context(|| format!("Failed to read {:?}.", &dfx_path))?;
        let mut config_json: Value = serde_json::from_slice(&content)
            .map_err(std::io::Error::from)
            .with_context(|| format!("Failed to parse {}.", dfx_path.to_string_lossy()))?;

        let frontend_value: serde_json::Map<String, Value> = [(
            "entrypoint".to_string(),
            ("src/".to_owned() + project_name_str + "_frontend/src/index.html").into(),
        )]
        .iter()
        .cloned()
        .collect();

        // Only update the dfx.json and install node dependencies if we're not running in dry run.
        if !dry_run {
            let assets_canister_json = config_json
                .pointer_mut(("/canisters/".to_owned() + project_name_str + "_frontend").as_str())
                .unwrap();
            assets_canister_json
                .as_object_mut()
                .unwrap()
                .insert("frontend".to_string(), Value::from(frontend_value));

            assets_canister_json
                .as_object_mut()
                .unwrap()
                .get_mut("source")
                .unwrap()
                .as_array_mut()
                .unwrap()
                .push(Value::from(
                    "dist/".to_owned() + project_name_str + "_frontend/",
                ));

            let pretty = serde_json::to_string_pretty(&config_json)
                .context("Invalid data: Cannot serialize configuration file.")?;
            std::fs::write(&dfx_path, pretty)
                .with_context(|| format!("Failed to write to {}.", dfx_path.to_string_lossy()))?;

            // Install node modules. Error is not blocking, we just show a message instead.
            if node_installed {
                let b = env.new_spinner("Installing node dependencies...".into());

                if npm_install(project_name)?.wait().is_ok() {
                    b.finish_with_message("Done.".into());
                } else {
                    b.finish_with_message(
                        "An error occurred. See the messages above for more details.".into(),
                    );
                }
            }
        }
    } else if !arg_frontend && !node_installed {
        warn!(
            log,
            "Node could not be found. Skipping installing the frontend example code."
        );
        warn!(
            log,
            "You can bypass this check by using the --frontend flag."
        );
    }

    Ok(())
}

fn get_agent_js_version_from_npm(dist_tag: &str) -> DfxResult<String> {
    std::process::Command::new("npm")
        .arg("show")
        .arg("@dfinity/agent")
        .arg(&format!("dist-tags.{}", dist_tag))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(DfxError::from)
        .and_then(|child| {
            let mut result = String::new();
            child
                .stdout
                .expect("Could not get the output of subprocess 'npm'.")
                .read_to_string(&mut result)
                .context("Failed to run 'npm'.")?;
            Ok(result.trim().to_string())
        })
}

pub fn exec(env: &dyn Environment, opts: NewOpts) -> DfxResult {
    let log = env.get_logger();
    let dry_run = opts.dry_run;
    let project_name = Path::new(opts.project_name.as_str());

    if project_name.exists() {
        bail!("Cannot create a new project because the directory already exists.");
    }

    let current_version = env.get_version();
    let version_str = format!("{}", current_version);

    // It is fine for the following command to timeout or fail. We
    // drop the error.
    let latest_version = get_latest_version(RELEASE_ROOT, Some(*CHECK_VERSION_TIMEOUT)).ok();

    if is_upgrade_necessary(latest_version.as_ref(), current_version) {
        warn_upgrade(log, latest_version.as_ref(), current_version);
    }

    env.get_cache().install()?;

    info!(
        log,
        r#"Creating new project "{}"..."#,
        project_name.display()
    );
    if dry_run {
        warn!(
            log,
            r#"Running in dry mode. Nothing will be committed to disk."#
        );
    }

    let project_name_str = project_name
        .to_str()
        .ok_or_else(|| anyhow!("Invalid argument: project_name"))?;

    let variables: BTreeMap<String, String> = [
        ("project_name".to_string(), project_name_str.to_string()),
        ("dfx_version".to_string(), version_str.clone()),
        ("dot".to_string(), ".".to_string()),
    ]
    .iter()
    .cloned()
    .collect();

    // Default to start with motoko
    let mut new_project_files = match opts.r#type.as_str() {
        "rust" => assets::new_project_rust_files().context("Failed to get rust archive.")?,
        "motoko" => assets::new_project_motoko_files().context("Failed to get motoko archive.")?,
        t => bail!("Unsupported canister type: {}", t),
    };
    write_files_from_entries(
        log,
        &mut new_project_files,
        project_name,
        dry_run,
        &variables,
    )?;

    scaffold_frontend_code(
        env,
        dry_run,
        project_name,
        opts.no_frontend,
        opts.frontend,
        &opts.agent_version,
        &variables,
    )?;

    if !dry_run {
        // If on mac, we should validate that XCode toolchain was installed.
        #[cfg(target_os = "macos")]
        {
            let mut should_git = true;
            if let Ok(code) = Command::new("xcode-select")
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
                init_git(log, project_name)?;
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            init_git(log, project_name)?;
        }

        if opts.r#type == "rust" {
            // dfx build will use --locked, so update the lockfile beforehand
            const MSG: &str = "You will need to run it yourself (or a similar command like `cargo vendor`), because `dfx build` will use the --locked flag with Cargo.";
            if let Ok(code) = Command::new("cargo")
                .arg("update")
                .arg("--manifest-path")
                .arg(project_name.join("Cargo.toml"))
                .stderr(Stdio::inherit())
                .stdout(Stdio::inherit())
                .status()
            {
                if !code.success() {
                    warn!(log, "Failed to run `cargo update`. {MSG}");
                }
            } else {
                warn!(
                    log,
                    "Failed to run `cargo update` - is Cargo installed? {MSG}"
                )
            }
        }
    }

    // Print welcome message.
    info!(
        log,
        // This needs to be included here because we cannot use the result of a function for
        // the format!() rule (and so it cannot be moved in the util::assets module).
        include_str!("../../assets/welcome.txt"),
        version_str,
        assets::dfinity_logo(),
        project_name_str
    );

    Ok(())
}

fn warn_upgrade(log: &Logger, latest_version: Option<&Version>, current_version: &Version) {
    warn!(log, "You seem to be running an outdated version of dfx.");

    let red = Style::new().red();
    let green = Style::new().green();
    let yellow = Style::new().yellow();

    let mut version_comparison =
        format!("Current version: {}", red.apply_to(current_version.clone()));
    if let Some(v) = latest_version {
        version_comparison += format!(
            "{} latest version: {}",
            yellow.apply_to(" → "),
            green.apply_to(v)
        )
        .as_str();
    }

    warn!(
        log,
        "\nYou are strongly encouraged to upgrade by running 'dfx upgrade'!"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_name_is_valid() {
        assert!(project_name_validator("a").is_ok());
        assert!(project_name_validator("a_").is_ok());
        assert!(project_name_validator("a_1").is_ok());
        assert!(project_name_validator("A").is_ok());
        assert!(project_name_validator("A1").is_ok());
        assert!(project_name_validator("a_good_name_").is_ok());
        assert!(project_name_validator("a_good_name").is_ok());
    }

    #[test]
    fn project_name_is_invalid() {
        assert!(project_name_validator("_a_good_name_").is_err());
        assert!(project_name_validator("__also_good").is_err());
        assert!(project_name_validator("_1").is_err());
        assert!(project_name_validator("_a").is_err());
        assert!(project_name_validator("1").is_err());
        assert!(project_name_validator("1_").is_err());
        assert!(project_name_validator("-").is_err());
        assert!(project_name_validator("_").is_err());
        assert!(project_name_validator("a-b-c").is_err());
        assert!(project_name_validator("🕹").is_err());
        assert!(project_name_validator("不好").is_err());
        assert!(project_name_validator("a:b").is_err());
    }
}
