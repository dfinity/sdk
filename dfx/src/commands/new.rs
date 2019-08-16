use crate::commands::{CliResult, CliError};
use crate::config::{Config, CONFIG_FILE_NAME};
use crate::util::logo::generate_logo;
use clap::{ArgMatches, SubCommand, Arg, App};
use std::io::Write;
use std::path::Path;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};


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
}

fn write_status(status: &str, color: Color, rest: &str) -> CliResult {
    let mut stream = StandardStream::stderr(ColorChoice::Auto);
    stream.set_color(ColorSpec::new().set_fg(Some(color)).set_bold(true))?;
    write!(&mut stream, "{:<12} ", status)?;
    stream.reset()?;
    writeln!(&mut stream, "{}", rest)?;

    Ok(())

    // TODO: color.
//    println!("{:<12} {}", status, rest);
}

pub fn create_file<P: AsRef<Path>>(path: P, content: &str, dry_run: bool) -> CliResult {
    let path = path.as_ref();
    if !dry_run {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(&path, content)?;
    }

    write_status("CREATE", Color::Green,
                 format!("{} ({} bytes)...", path.to_str().unwrap(), content.len()).as_str())?;
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

    println!(r#"Creating new project "{}"..."#, project_name.to_str().unwrap());
    if dry_run {
        println!(r#"Running in dry mode. Nothing will be committed to disk."#);
    }

    create_file(project_name.join(CONFIG_FILE_NAME), "{}", dry_run)?;
    create_file(project_name.join("src").join("main.as"), "This File Kept Empty", dry_run)?;
    create_file(project_name.join("README.md"), format!("# Welcome to {}", project_name.display()).as_str(), dry_run)?;
    create_dir(project_name.join("bin"), dry_run)?;

    // Print welcome message.
    println!(include_str!("../../messages/welcome.txt"), generate_logo(), project_name.to_str().unwrap());

    Ok(())
}
