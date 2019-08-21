use crate::commands::{CliResult, CliError};
use crate::config::{Config, CONFIG_FILE_NAME};
use crate::util::FakeProgress;
use crate::util::logo::generate_logo;
use clap::{ArgMatches, SubCommand, Arg, App};
use console::{style, Color, Term};
use indicatif::{HumanBytes, ProgressStyle};
use std::path::Path;

// This is easier to use.
macro_rules! asset_str {
    ($file:expr) => (include_str!(concat!("../../assets/", $file)));
    ($file:expr,) => (asset_str!($file));
}

pub fn available() -> bool {
    !Config::from_current_dir().is_ok()
}

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("new")
        .about("Create a new DFINITY project.")
        .arg(
            Arg::with_name("project_name")
                .help("The name of the project to create.")
                .required(true)
        )
        .arg(
            Arg::with_name("dry_run")
                .help("Do not commit anything to the file system.")
                .long("dry-run")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("dfx_version")
                .help("Force a version of DFX to use in the new project.")
                .long("dfx-version")
                .takes_value(true)
        )
}

fn write_status(status: &str, color: Color, rest: &str) -> CliResult {
    Term::stderr().write_line(
        format!("{:<12} {}", style(status).fg(color).bold().to_owned(), rest).as_str(),
    )?;

    Ok(())
}

pub fn create_file<P: AsRef<Path>>(path: P, content: &str, dry_run: bool) -> CliResult {
    let path = path.as_ref();
    if !dry_run {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(&path, content)?;
    }

    let size = content.len() as u64;
    write_status("CREATE", Color::Green,
                 format!("{} ({})...", path.to_str().unwrap(), HumanBytes(size)).as_str())?;
    Ok(())
}

pub fn create_dir<P: AsRef<Path>>(path: P, dry_run: bool) -> CliResult {
    let path = path.as_ref();
    if path.is_dir() {
        return Ok(())
    }

    if !dry_run {
        std::fs::create_dir_all(&path)?;
    }

    write_status("CREATE_DIR", Color::Blue, path.to_str().unwrap())?;
    Ok(())
}

pub fn exec(args: &ArgMatches<'_>) -> CliResult {
    let dry_run = args.is_present("dry_run");
    let project_name = Path::new(args.value_of("project_name").unwrap());

    // Make sure we don't embed a project in another project.
    if let Ok(config_path) = Config::resolve_config_path(&std::env::current_dir()?) {
        return Err(CliError::new(
            failure::format_err!(
                "Config file found at {}. Are you already in a DFINITY project?",
                config_path.to_str().unwrap(),
            ),
            1,
        ));
    }

    if project_name.exists() {
        return Err(CliError::new(
            failure::format_err!("Project directory already exists."),
            1,
        ));
    }

    let mut p = FakeProgress::new();
    p.add(
        600..1200,
        |b| {
            b.set_style(ProgressStyle::default_spinner());
            b.set_message("Looking for latest version...");
        },
        |b| b.finish_with_message("Latest version already installed."),
    );
    p.join();

    println!();
    println!(r#"Creating new project "{}"..."#, project_name.to_str().unwrap());
    if dry_run {
        println!(r#"Running in dry mode. Nothing will be committed to disk."#);
    }

    create_file(project_name.join(CONFIG_FILE_NAME), asset_str!("files/dfinity.json"), dry_run)?;
    create_file(project_name.join("cannisters").join("hello").join("main.as"),
                asset_str!("files/cannisters/hello/main.as"),
                dry_run)?;
    create_file(project_name.join("src").join("ask_hello.js"), "// TODO: this file\n", dry_run)?;
    create_file(project_name.join("README.md"),
                format!(asset_str!("files/README.md"), project = project_name.display()).as_str(),
                dry_run)?;
    create_dir(project_name.join("bin"), dry_run)?;

    let version = "v0.1.0";

    // Print welcome message.
    println!(asset_str!("welcome.txt"), version, generate_logo(), project_name.to_str().unwrap());

    Ok(())
}
