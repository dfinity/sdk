use crate::lib::env::{BinaryCacheEnv, PlatformEnv, VersionEnv};
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::assets;
use clap::{App, Arg, ArgMatches, SubCommand};
use console::style;
use indicatif::HumanBytes;
use std::io::Read;
use std::path::{Path, PathBuf};

const DRY_RUN: &str = "dry_run";
const PROJECT_NAME: &str = "project_name";

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("new")
        .about(UserMessage::CreateProject.to_str())
        .arg(
            Arg::with_name(PROJECT_NAME)
                .help(UserMessage::ProjectName.to_str())
                .required(true),
        )
        .arg(
            Arg::with_name(DRY_RUN)
                .help(UserMessage::DryRun.to_str())
                .long("dry-run")
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

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryCacheEnv + PlatformEnv + VersionEnv,
{
    let dry_run = args.is_present(DRY_RUN);
    let project_name_path = args
        .value_of(PROJECT_NAME)
        .ok_or_else(|| DfxError::InvalidArgument("project path".to_string()))?;
    let project_name = Path::new(project_name_path);

    if project_name.exists() {
        return Err(DfxError::ProjectExists);
    }

    let dfx_version = env.get_version();

    eprintln!(r#"Creating new project "{}"..."#, project_name.display());
    if dry_run {
        eprintln!(r#"Running in dry mode. Nothing will be committed to disk."#);
    }

    let mut new_project_files = assets::new_project_files()?;
    let project_name_str = project_name
        .to_str()
        .ok_or_else(|| DfxError::InvalidArgument("project name".to_string()))?;

    for file in new_project_files.entries()? {
        let mut file = file?;

        if file.header().entry_type().is_dir() {
            continue;
        }

        let mut s = String::new();
        file.read_to_string(&mut s)
            .or_else(|e| Err(DfxError::Io(e)))?;

        // Perform replacements.
        let s = s.replace("{project_name}", project_name_str);
        let s = s.replace("{dfx_version}", dfx_version);

        // Perform path replacements.
        let p = PathBuf::from(
            project_name
                .join(file.header().path()?)
                .to_str()
                .ok_or_else(|| {
                    DfxError::InvalidArgument("project name path or file header".to_string())
                })?
                .replace("__dot__", ".")
                .as_str(),
        );

        create_file(p.as_path(), s.as_str(), dry_run)?;
    }

    if !dry_run {
        // Check that git is available.
        let init_status = std::process::Command::new("git")
            .arg("init")
            .current_dir(&project_name)
            .status();

        if let Ok(s) = init_status {
            if s.success() {
                eprintln!("Creating git repository...");
                std::process::Command::new("git")
                    .arg("add")
                    .current_dir(&project_name)
                    .arg(".")
                    .output()?;
                std::process::Command::new("git")
                    .arg("commit")
                    .current_dir(&project_name)
                    .arg("-a")
                    .arg("--message=Initial commit.")
                    .output()?;
            }
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
