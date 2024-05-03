use crate::config::cache::DiskBasedCache;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::info::replica_rev;
use crate::lib::manifest::{get_latest_version, is_upgrade_necessary};
use crate::lib::program;
use crate::util::assets;
use crate::util::clap::parsers::project_name_parser;
use anyhow::{anyhow, bail, ensure, Context};
use clap::{Parser, ValueEnum};
use console::{style, Style};
use dfx_core::json::{load_json_file, save_json_file};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{FuzzySelect, MultiSelect};
use fn_error_context::context;
use indicatif::HumanBytes;
use semver::Version;
use slog::{info, warn, Logger};
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tar::Archive;

// const DRY_RUN: &str = "dry_run";
// const PROJECT_NAME: &str = "project_name";
const RELEASE_ROOT: &str = "https://sdk.dfinity.org";

// The dist-tag to use when getting the version from NPM.
const AGENT_JS_DEFAULT_INSTALL_DIST_TAG: &str = "latest";

// Tested on a phone tethering connection. This should be fine with
// little impact to the user, given that "new" is supposedly a
// heavy-weight operation. Thus, worst case we are utilizing the user
// expectation for the duration to have a more expensive version
// check.
const CHECK_VERSION_TIMEOUT: Duration = Duration::from_secs(2);

/// Creates a new project.
#[derive(Parser)]
pub struct NewOpts {
    /// Specifies the name of the project to create.
    #[arg(value_parser = project_name_parser)]
    project_name: String,

    /// Choose the type of canister in the starter project.
    #[arg(long, value_enum)]
    r#type: Option<BackendType>,

    /// Provides a preview the directories and files to be created without adding them to the file system.
    #[arg(long)]
    dry_run: bool,

    /// Choose the type of frontend in the starter project. Defaults to vanilla.
    #[arg(long, value_enum, default_missing_value = "vanilla")]
    frontend: Option<FrontendType>,

    /// Skip installing the frontend code example.
    #[arg(long, conflicts_with = "frontend")]
    no_frontend: bool,

    /// Overrides which version of the JavaScript Agent to install. By default, will contact
    /// NPM to decide.
    #[arg(long, requires("frontend"))]
    agent_version: Option<String>,

    #[arg(long, value_enum)]
    extras: Vec<Extra>,
}

#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq)]
enum BackendType {
    Motoko,
    Rust,
    Azle,
    Kybra,
}

impl Display for BackendType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Motoko => "Motoko",
            Self::Rust => "Rust",
            Self::Azle => "TypeScript (Azle)",
            Self::Kybra => "Python (Kybra)",
        }
        .fmt(f)
    }
}

#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq)]
enum FrontendType {
    #[value(name = "sveltekit")]
    SvelteKit,
    Vanilla,
    Vue,
    React,
    SimpleAssets,
    None,
}

impl Display for FrontendType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SvelteKit => "SvelteKit",
            Self::Vanilla => "Vanilla JS",
            Self::Vue => "Vue",
            Self::React => "React",
            Self::SimpleAssets => "No JS template",
            Self::None => "No frontend canister",
        }
        .fmt(f)
    }
}

impl FrontendType {
    fn has_js(&self) -> bool {
        !matches!(self, Self::None | Self::SimpleAssets)
    }
}

#[derive(ValueEnum, Debug, Copy, Clone, PartialEq, Eq)]
enum Extra {
    InternetIdentity,
    Bitcoin,
    FrontendTests,
}

impl Display for Extra {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InternetIdentity => "Internet Identity",
            Self::Bitcoin => "Bitcoin (Regtest)",
            Self::FrontendTests => "Frontend tests",
        }
        .fmt(f)
    }
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
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write to {}.", path.to_string_lossy()))?;
    }

    info!(log, "{}", Status::Create(path, content.len()));
    Ok(())
}

fn json_patch_file(
    _log: &Logger,
    patch_path: &Path,
    patch_content: &[u8],
    dry_run: bool,
) -> DfxResult {
    if !dry_run {
        let patch: json_patch::Patch = serde_json::from_slice(patch_content)
            .with_context(|| format!("Failed to parse {}", patch_path.display()))?;
        let to_patch = patch_path.with_extension("json");
        ensure!(
            to_patch.exists(),
            "Failed to patch {}: not found",
            to_patch.display()
        );
        let mut value = load_json_file(&to_patch)?;
        json_patch::patch(&mut value, &patch)
            .with_context(|| format!("Failed to patch {}", to_patch.display()))?;
        save_json_file(&to_patch, &value)?;
    }
    Ok(())
}

fn patch_file(_log: &Logger, patch_path: &Path, patch_content: &[u8], dry_run: bool) -> DfxResult {
    if !dry_run {
        let patch_content = std::str::from_utf8(patch_content)
            .with_context(|| format!("Failed to parse {}", patch_path.display()))?;
        let patch = patch::Patch::from_single(patch_content)
            .map_err(|e| anyhow!("Failed to parse {}: {e}", patch_path.display()))?;
        let to_patch = patch_path.with_extension("");
        let existing_content = dfx_core::fs::read_to_string(&to_patch)?;
        let patched_content = apply_patch::apply_to(&patch, &existing_content)
            .with_context(|| format!("Failed to patch {}", to_patch.display()))?;
        dfx_core::fs::write(&to_patch, patched_content)?;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn create_dir<P: AsRef<Path>>(log: &Logger, path: P, dry_run: bool) -> DfxResult {
    let path = path.as_ref();
    if path.is_dir() {
        return Ok(());
    }

    if !dry_run {
        std::fs::create_dir_all(path)
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
        if p.extension() == Some("json-patch".as_ref()) {
            json_patch_file(log, &p, &v, dry_run)?;
        } else if p.extension() == Some("patch".as_ref()) {
            patch_file(log, &p, &v, dry_run)?;
        } else {
            create_file(log, p.as_path(), &v, dry_run)?;
        }
    }

    Ok(())
}

#[context("Failed to run 'npm install'.")]
fn npm_install(location: &Path) -> DfxResult<std::process::Child> {
    Command::new(program::NPM)
        .arg("install")
        .arg("--quiet")
        .arg("--no-progress")
        .arg("--workspaces")
        .arg("--if-present")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .current_dir(location)
        .spawn()
        .map_err(DfxError::from)
}

#[context("Failed to scaffold frontend code.")]
fn scaffold_frontend_code(
    env: &dyn Environment,
    dry_run: bool,
    project_name: &Path,
    frontend: FrontendType,
    extras: &[Extra],
    agent_version: &Option<String>,
    variables: &BTreeMap<String, String>,
) -> DfxResult {
    let log = env.get_logger();
    let node_installed = program_installed(program::NODE);
    let npm_installed = program_installed(program::NPM);

    let project_name_str = project_name
        .to_str()
        .ok_or_else(|| anyhow!("Invalid argument: project_name"))?;

    // Check if node and npm are available, and if so create the files for the frontend build.
    if node_installed && npm_installed {
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

        let mut new_project_files = match frontend {
            FrontendType::Vanilla => assets::new_project_vanillajs_files(),
            FrontendType::SvelteKit => assets::new_project_svelte_files(),
            FrontendType::Vue => assets::new_project_vue_files(),
            FrontendType::React => assets::new_project_react_files(),
            FrontendType::SimpleAssets => assets::new_project_assets_files(),
            FrontendType::None => unreachable!(),
        }?;
        write_files_from_entries(
            log,
            &mut new_project_files,
            project_name,
            dry_run,
            &variables,
        )?;
        if extras.contains(&Extra::FrontendTests) {
            let mut test_files = match frontend {
                FrontendType::SvelteKit => assets::new_project_svelte_test_files(),
                FrontendType::React => assets::new_project_react_test_files(),
                FrontendType::Vue => assets::new_project_vue_test_files(),
                FrontendType::Vanilla => assets::new_project_vanillajs_test_files(),
                FrontendType::SimpleAssets => {
                    bail!("Cannot add frontend tests to --frontend-type simple-assets")
                }
                FrontendType::None => bail!("Cannot add frontend tests to --no-frontend"),
            }?;
            write_files_from_entries(log, &mut test_files, project_name, dry_run, &variables)?;
        }

        // Only install node dependencies if we're not running in dry run.
        if !dry_run && frontend != FrontendType::SimpleAssets {
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
    } else {
        if !node_installed {
            warn!(
                log,
                "Node could not be found. Skipping installing the frontend example code."
            );
        }
        if !npm_installed {
            warn!(
                log,
                "npm could not be found. Skipping installing the frontend example code."
            );
        }

        warn!(
            log,
            "You can bypass this check by using the --frontend flag."
        );
        write_files_from_entries(
            log,
            &mut assets::new_project_assets_files()?,
            project_name,
            dry_run,
            variables,
        )?;
    }

    Ok(())
}

fn program_installed(program: &str) -> bool {
    let result = Command::new(program).arg("--version").output();
    matches!(result, Ok(output) if output.status.success())
}

fn get_agent_js_version_from_npm(dist_tag: &str) -> DfxResult<String> {
    let output = Command::new(program::NPM)
        .arg("show")
        .arg("@dfinity/agent")
        .arg(&format!("dist-tags.{}", dist_tag))
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .context("Failed to execute 'npm show @dfinity/agent'")?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "npm command failed with exit code {}",
            output.status.code().unwrap_or_default()
        ));
    }
    let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(result)
}

pub fn exec(env: &dyn Environment, mut opts: NewOpts) -> DfxResult {
    use BackendType::*;
    let log = env.get_logger();
    let dry_run = opts.dry_run;

    let r#type = if let Some(r#type) = opts.r#type {
        r#type
    } else if opts.frontend.is_none() && opts.extras.is_empty() && io::stdout().is_terminal() {
        opts = get_opts_interactively(opts)?;
        opts.r#type.unwrap()
    } else {
        Motoko
    };

    let project_name = Path::new(opts.project_name.as_str());
    if project_name.exists() {
        bail!("Cannot create a new project because the directory already exists.");
    }

    let current_version = env.get_version();
    let version_str = format!("{}", current_version);

    // It is fine for the following command to timeout or fail. We
    // drop the error.
    let latest_version = get_latest_version(RELEASE_ROOT, Some(CHECK_VERSION_TIMEOUT)).ok();

    if is_upgrade_necessary(latest_version.as_ref(), current_version) {
        warn_upgrade(log, latest_version.as_ref(), current_version);
    }

    DiskBasedCache::install(&env.get_cache().version_str())?;

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

    let (backend_name, frontend_name) = if project_name_str.contains('-') {
        (
            format!("{project_name_str}-backend"),
            format!("{project_name_str}-frontend"),
        )
    } else {
        (
            format!("{project_name_str}_backend"),
            format!("{project_name_str}_frontend"),
        )
    };

    let variables: BTreeMap<String, String> = BTreeMap::from([
        ("project_name".to_string(), project_name_str.to_string()),
        (
            "project_name_ident".to_string(),
            project_name_str.replace('-', "_"),
        ),
        ("backend_name".to_string(), backend_name.clone()),
        (
            "backend_name_ident".to_string(),
            backend_name.replace('-', "_"),
        ),
        ("frontend_name".to_string(), frontend_name.clone()),
        (
            "frontend_name_ident".to_string(),
            frontend_name.replace('-', "_"),
        ),
        ("dfx_version".to_string(), version_str.clone()),
        ("dot".to_string(), ".".to_string()),
        ("ic_commit".to_string(), replica_rev().to_string()),
    ]);

    write_files_from_entries(
        log,
        &mut assets::new_project_base_files().context("Failed to get base project archive.")?,
        project_name,
        dry_run,
        &variables,
    )?;

    let frontend = if opts.no_frontend {
        FrontendType::None
    } else {
        opts.frontend.unwrap_or(FrontendType::Vanilla)
    };

    if r#type == Azle || frontend.has_js() {
        write_files_from_entries(
            log,
            &mut assets::new_project_js_files().context("Failed to get JS config archive.")?,
            project_name,
            dry_run,
            &variables,
        )?;
    }

    // Default to start with motoko
    let mut new_project_files = match r#type {
        Rust => assets::new_project_rust_files().context("Failed to get rust archive.")?,
        Motoko => assets::new_project_motoko_files().context("Failed to get motoko archive.")?,
        Azle => assets::new_project_azle_files().context("Failed to get azle archive.")?,
        Kybra => assets::new_project_kybra_files().context("Failed to get kybra archive.")?,
    };
    write_files_from_entries(
        log,
        &mut new_project_files,
        project_name,
        dry_run,
        &variables,
    )?;

    if opts.extras.contains(&Extra::InternetIdentity) {
        write_files_from_entries(
            log,
            &mut assets::new_project_internet_identity_files()?,
            project_name,
            dry_run,
            &variables,
        )?;
    }
    if opts.extras.contains(&Extra::Bitcoin) {
        write_files_from_entries(
            log,
            &mut assets::new_project_bitcoin_files()?,
            project_name,
            dry_run,
            &variables,
        )?;
    }
    if frontend != FrontendType::None {
        scaffold_frontend_code(
            env,
            dry_run,
            project_name,
            frontend,
            &opts.extras,
            &opts.agent_version,
            &variables,
        )?;
    }

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

        if r#type == Rust {
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

fn get_opts_interactively(opts: NewOpts) -> DfxResult<NewOpts> {
    use BackendType::*;
    use Extra::*;
    use FrontendType::*;
    let theme = ColorfulTheme::default();
    let backends_list = [Motoko, Rust, Azle, Kybra];
    let backend = FuzzySelect::with_theme(&theme)
        .items(&backends_list)
        .default(0)
        .with_prompt("Select a backend language:")
        .interact()?;
    let backend = backends_list[backend];
    let frontends_list = [SvelteKit, React, Vue, Vanilla, SimpleAssets, None];
    let frontend = FuzzySelect::with_theme(&theme)
        .items(&frontends_list)
        .default(0)
        .with_prompt("Select a frontend framework:")
        .interact()?;
    let frontend = frontends_list[frontend];
    let mut extras_list = vec![InternetIdentity, Bitcoin];
    if frontend != None && frontend != SimpleAssets {
        extras_list.push(FrontendTests);
    }
    let extras = MultiSelect::with_theme(&theme)
        .items(&extras_list)
        .with_prompt("Add extra features (space to select, enter to confirm)")
        .interact()?;

    let extras = extras.into_iter().map(|x| extras_list[x]).collect();

    let opts = NewOpts {
        extras,
        frontend: Some(frontend),
        r#type: Some(backend),
        ..opts
    };
    Ok(opts)
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
        assert!(project_name_parser("a").is_ok());
        assert!(project_name_parser("a_").is_ok());
        assert!(project_name_parser("a_1").is_ok());
        assert!(project_name_parser("A").is_ok());
        assert!(project_name_parser("A1").is_ok());
        assert!(project_name_parser("a_good_name_").is_ok());
        assert!(project_name_parser("a_good_name").is_ok());
    }

    #[test]
    fn project_name_is_invalid() {
        assert!(project_name_parser("_a_good_name_").is_err());
        assert!(project_name_parser("__also_good").is_err());
        assert!(project_name_parser("_1").is_err());
        assert!(project_name_parser("_a").is_err());
        assert!(project_name_parser("1").is_err());
        assert!(project_name_parser("1_").is_err());
        assert!(project_name_parser("-").is_err());
        assert!(project_name_parser("_").is_err());
        assert!(project_name_parser("🕹").is_err());
        assert!(project_name_parser("不好").is_err());
        assert!(project_name_parser("a:b").is_err());
    }
}
