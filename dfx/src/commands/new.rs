use crate::lib::env::{BinaryCacheEnv, PlatformEnv, VersionEnv};
use crate::lib::error::{DfxError, DfxResult};
use crate::util::assets;
use clap::{App, Arg, ArgMatches, SubCommand};
use console::{style, Color, Term};
use indicatif::{HumanBytes, ProgressBar, ProgressDrawTarget};
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("new")
        .about("Create a new project.")
        .arg(
            Arg::with_name("project_name")
                .help("The name of the project to create.")
                .required(true),
        )
        .arg(
            Arg::with_name("dry_run")
                .help("Do not write anything to the file system.")
                .long("dry-run")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("dfx_version")
                .help("Specify the version of DFX to use in the new project.")
                .long("dfx-version")
                .takes_value(true),
        )
}

fn write_status(status: &str, color: Color, rest: &str) -> DfxResult {
    Term::stderr().write_line(
        format!("{:<12} {}", style(status).fg(color).bold().to_owned(), rest).as_str(),
    )?;

    Ok(())
}

pub fn create_file(path: &Path, content: &str, dry_run: bool) -> DfxResult {
    if !dry_run {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::write(&path, content)?;
    }

    let size = content.len() as u64;
    write_status(
        "CREATE",
        Color::Green,
        format!("{} ({})...", path.to_str().unwrap(), HumanBytes(size)).as_str(),
    )?;
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

    write_status("CREATE_DIR", Color::Blue, path.to_str().unwrap())?;
    Ok(())
}

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryCacheEnv + PlatformEnv + VersionEnv,
{
    let dry_run = args.is_present("dry_run");
    let project_name = Path::new(args.value_of("project_name").unwrap());

    if project_name.exists() {
        return Err(DfxError::Unknown(
            "Project directory already exists.".to_owned(),
        ));
    }

    let dfx_version = env.get_version();
    let b = ProgressBar::new_spinner();
    b.set_draw_target(ProgressDrawTarget::stderr());
    b.set_message("Looking for latest version...");
    b.enable_steady_tick(80);

    std::thread::sleep(std::time::Duration::from_secs(1));
    if !env.is_installed().unwrap() {
        env.install().unwrap();
        b.finish_with_message(
            format!("Version v{} installed successfully.", env.get_version()).as_str(),
        );
    } else {
        b.finish_with_message(
            format!("Version v{} already installed.", env.get_version()).as_str(),
        );
    }

    eprintln!(
        r#"Creating new project "{}"..."#,
        project_name.to_str().unwrap()
    );
    if dry_run {
        eprintln!(r#"Running in dry mode. Nothing will be committed to disk."#);
    }

    for file in assets::new_project_files().unwrap().entries()? {
        let mut file = file?;

        if file.header().entry_type().is_dir() {
            continue;
        }

        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();

        // Perform replacements.
        let s = s.replace("{project_name}", project_name.to_str().unwrap());
        let s = s.replace("{dfx_version}", dfx_version);

        // Perform path replacements.
        let p = PathBuf::from(
            project_name
                .join(file.header().path()?)
                .to_str()
                .unwrap()
                .replace("__dot__", ".")
                .as_str(),
        );

        create_file(p.as_path(), s.as_str(), dry_run)?;
    }

    if !dry_run {
        eprintln!("Creating git repository...");
        std::process::Command::new("git")
            .arg("init")
            .current_dir(&project_name)
            .output()?;
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

    // Print welcome message.
    eprintln!(
        // This needs to be included here because we cannot use the result of a function for
        // the format!() rule (and so it cannot be moved in the util::assets module).
        include_str!("../../assets/welcome.txt"),
        dfx_version,
        assets::color_logo(),
        project_name.to_str().unwrap()
    );

    Ok(())
}
