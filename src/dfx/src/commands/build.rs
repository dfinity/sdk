use crate::config::dfinity::{Config, Profile};
use crate::config::{cache, dfx_version};
use crate::lib::canister_info::CanisterInfo;
use crate::lib::env::{BinaryResolverEnv, ProjectConfigEnv};
use crate::lib::error::DfxError::BuildError;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::assets;
use clap::{App, ArgMatches, SubCommand};
use ic_http_agent::CanisterId;
use indicatif::{ProgressBar, ProgressDrawTarget};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::path::Path;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build").about(UserMessage::BuildCanister.to_str())
}

fn get_asset_fn(content: &HashMap<String, String>) -> String {
    // Create the if/else series.
    let mut if_else = String::new();
    content.iter().for_each(|(filename, content)| {
        if_else += format!(
            r#"if (path == "{}") {par} return "{}"; {end};{endline}"#,
            filename,
            content
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", ""),
            par = "{",
            end = "}",
            endline = "\n"
        )
        .as_str();
    });

    format!(
        r#"
            public query func __dfx_asset_path(path: Text): async Text {par}
              {}
              return "";
            {end};
        "#,
        if_else,
        par = "{",
        end = "}"
    )
}

/// Compile a motoko file.
fn motoko_compile<T: BinaryResolverEnv>(
    env: &T,
    profile: Option<Profile>,
    content: &str,
    input_path: &Path,
    output_path: &Path,
    assets: &HashMap<String, String>,
) -> DfxResult {
    // Invoke the compiler in debug (development) or release mode, based on the current profile:
    let arg_profile = match profile {
        Some(Profile::Release) => "--release",
        _ => "--debug",
    };

    let mo_rts_path = env.get_binary_command_path("mo-rts.wasm")?;
    let stdlib_path = env.get_binary_command_path("stdlib")?;

    let mut content = content.to_string();
    // Because we don't have an AST (yet) we need to do some regex magic.
    // Find `actor {`
    let re = regex::Regex::new(r"\bactor\b.*?\{")
        .map_err(|_| DfxError::UnknownCommand("".to_string()))?;
    if let Some(actor_idx) = re.find(&content) {
        let (before, after) = content.split_at(actor_idx.end());
        content = before.to_string() + get_asset_fn(assets).as_str() + after;
    }

    let working_dir = input_path
        .parent()
        .ok_or_else(|| DfxError::Unknown("Cannot compile root.".to_string()))?;
    let working_dir = if working_dir.exists() {
        working_dir.to_path_buf()
    } else {
        std::env::current_dir()?
    };

    let mut process = env
        .get_binary_command("moc")?
        .env("MOC_RTS", mo_rts_path.as_path())
        .current_dir(&working_dir)
        .stdin(std::process::Stdio::piped())
        .arg("-c")
        .arg("/dev/stdin")
        .arg(arg_profile)
        .arg("-o")
        .arg(&output_path)
        .arg("--package")
        .arg("stdlib")
        .arg(&stdlib_path.as_path())
        .spawn()?;

    if let Some(ref mut stdin) = process.stdin {
        stdin.write_all(content.as_bytes())?;
    } else {
        return Err(DfxError::BuildError(BuildErrorKind::MotokoCompilerError(
            "No STDIN???!?".to_string(),
        )));
    }
    let output = process.wait_with_output()?;

    if !output.status.success() {
        Err(DfxError::BuildError(BuildErrorKind::MotokoCompilerError(
            // We choose to join the strings and not the vector in case there is a weird
            // incorrect character at the end of stdout.
            String::from_utf8_lossy(&output.stdout).to_string()
                + &String::from_utf8_lossy(&output.stderr),
        )))
    } else {
        Ok(())
    }
}

fn didl_compile<T: BinaryResolverEnv>(env: &T, input_path: &Path, output_path: &Path) -> DfxResult {
    let stdlib_path = env.get_binary_command_path("stdlib")?;

    let output = env
        .get_binary_command("moc")?
        .arg("--idl")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--package")
        .arg("stdlib")
        .arg(&stdlib_path.as_path())
        .output()?;

    if !output.status.success() {
        Err(DfxError::BuildError(BuildErrorKind::IdlGenerationError(
            String::from_utf8_lossy(&output.stdout).to_string()
                + &String::from_utf8_lossy(&output.stderr),
        )))
    } else {
        Ok(())
    }
}

fn build_did_js<T: BinaryResolverEnv>(env: &T, input_path: &Path, output_path: &Path) -> DfxResult {
    let output = env
        .get_binary_command("didc")?
        .arg("--js")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()?;

    if !output.status.success() {
        Err(DfxError::BuildError(BuildErrorKind::DidJsGenerationError(
            String::from_utf8_lossy(&output.stdout).to_string()
                + &String::from_utf8_lossy(&output.stderr),
        )))
    } else {
        Ok(())
    }
}

fn build_canister_js(canister_id: &CanisterId, canister_info: &CanisterInfo) -> DfxResult {
    let output_root = canister_info.get_output_root();
    let output_canister_js_path = canister_info.get_output_canister_js_path();

    let mut language_bindings = assets::language_bindings()?;
    let mut build_assets = assets::build_assets()?;

    let mut file = language_bindings.entries()?.next().unwrap()?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    let new_file_contents = file_contents
        .replace("{canister_id}", &canister_id.to_hex())
        .replace("{project_name}", canister_info.get_name());

    let output_canister_js_path_str = output_canister_js_path.to_str().ok_or_else(|| {
        DfxError::BuildError(BuildErrorKind::CanisterJsGenerationError(format!(
            "Unable to convert output canister js path to a string: {:#?}",
            output_canister_js_path
        )))
    })?;
    std::fs::write(output_canister_js_path_str, new_file_contents)?;

    if canister_info.has_frontend() {
        for entry in build_assets.entries()? {
            let mut file = entry?;

            if file.header().entry_type().is_dir() {
                continue;
            }

            let mut file_contents = String::new();
            file.read_to_string(&mut file_contents)?;
            if let Some(p) = output_root.join(file.header().path()?).parent() {
                std::fs::create_dir_all(&p)?;
            }
            std::fs::write(&output_root.join(file.header().path()?), file_contents)?;
        }
    }

    Ok(())
}

fn build_file<T>(
    env: &T,
    config: &Config,
    name: &str,
    assets: &HashMap<String, String>,
) -> DfxResult
where
    T: BinaryResolverEnv,
{
    let canister_info = CanisterInfo::load(config, name).map_err(|_| {
        BuildError(BuildErrorKind::CanisterNameIsNotInConfigError(
            name.to_owned(),
        ))
    })?;
    let config = config.get_config();
    let profile = config.profile.clone();
    let input_path = canister_info.get_main_path();

    let output_wasm_path = canister_info.get_output_wasm_path();
    match input_path.extension().and_then(OsStr::to_str) {
        // TODO(SDK-441): Revisit supporting compilation from WAT files.
        Some("wat") => {
            let wat = std::fs::read(input_path)?;
            let wasm = wabt::wat2wasm(wat)
                .map_err(|e| DfxError::BuildError(BuildErrorKind::WatCompileError(e)))?;

            std::fs::create_dir_all(canister_info.get_output_root())?;
            std::fs::write(&output_wasm_path, wasm)?;

            // Write the CID.
            std::fs::write(
                canister_info.get_canister_id_path(),
                canister_info.generate_canister_id()?.into_blob().0,
            )
            .map_err(DfxError::from)?;

            Ok(())
        }

        Some("mo") => {
            let output_idl_path = canister_info.get_output_idl_path();
            let output_did_js_path = canister_info.get_output_did_js_path();

            std::fs::create_dir_all(canister_info.get_output_root())?;
            let canister_id = canister_info.generate_canister_id()?;

            let content = std::fs::read_to_string(input_path)?;
            motoko_compile(
                env,
                profile,
                &content,
                &input_path,
                &output_wasm_path,
                assets,
            )?;
            didl_compile(env, &input_path, &output_idl_path)?;
            build_did_js(env, &output_idl_path, &output_did_js_path)?;
            build_canister_js(&canister_id, &canister_info)?;

            // Write the CID.
            std::fs::write(
                canister_info.get_canister_id_path(),
                canister_id.into_blob().0,
            )
            .map_err(DfxError::from)?;

            Ok(())
        }
        Some(ext) => Err(DfxError::BuildError(BuildErrorKind::InvalidExtension(
            ext.to_owned(),
        ))),
        None => Err(DfxError::BuildError(BuildErrorKind::InvalidExtension(
            "".to_owned(),
        ))),
    }?;

    Ok(())
}

pub fn exec<T>(env: &T, _args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryResolverEnv + ProjectConfigEnv,
{
    // Read the config.
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    cache::install_version(dfx_version(), false)?;

    let build_stage_bar = ProgressBar::new_spinner();
    build_stage_bar.set_draw_target(ProgressDrawTarget::stderr());
    build_stage_bar.set_message("Building canisters...");
    build_stage_bar.enable_steady_tick(80);

    let maybe_canisters = &config.get_config().canisters;
    if maybe_canisters.is_none() {
        build_stage_bar.finish_with_message("No canisters, nothing to build.");
        return Ok(());
    }
    let canisters = maybe_canisters.as_ref().unwrap();

    for name in canisters.keys() {
        build_stage_bar.set_message(&format!("Building canister {}...", name));
        match build_file(env, &config, name, &HashMap::new()) {
            Ok(_) => {}
            Err(e) => {
                build_stage_bar
                    .finish_with_message(&format!(r#"Failed to build canister "{}":"#, name));
                eprintln!("{:?}", e);
                return Err(e);
            }
        }
    }
    build_stage_bar.finish_with_message("Done building canisters...");

    // If there is not a package.json, we don't have a frontend and can quit early.
    if !config.get_project_root().join("package.json").exists() {
        return Ok(());
    }

    let build_stage_bar = ProgressBar::new_spinner();
    build_stage_bar.set_draw_target(ProgressDrawTarget::stderr());
    build_stage_bar.set_message("Building frontend...");
    build_stage_bar.enable_steady_tick(80);

    let mut process = std::process::Command::new("npm")
        .arg("run")
        .arg("build")
        .env("DFX_VERSION", &dfx_version())
        .current_dir(config.get_project_root())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let status = process.wait()?;

    if !status.success() {
        let mut str = String::new();
        process.stderr.unwrap().read_to_string(&mut str)?;
        eprintln!("NPM failed to run:\n{}", str);
        return Err(DfxError::BuildError(BuildErrorKind::FrontendBuildError()));
    }

    build_stage_bar.finish_with_message("Done building frontend...");

    let build_stage_bar = ProgressBar::new_spinner();
    build_stage_bar.set_draw_target(ProgressDrawTarget::stderr());
    build_stage_bar.set_message("Bundling frontend assets in the canister...");
    build_stage_bar.enable_steady_tick(80);

    let frontends: Vec<String> = canisters
        .iter()
        .filter(|(_, v)| v.frontend.is_some())
        .map(|(k, _)| k)
        .cloned()
        .collect();
    for name in frontends {
        let canister_info = CanisterInfo::load(config, name.as_str()).map_err(|_| {
            BuildError(BuildErrorKind::CanisterNameIsNotInConfigError(
                name.to_owned(),
            ))
        })?;

        let mut assets: HashMap<String, String> = HashMap::new();
        for dir_entry in std::fs::read_dir(canister_info.get_output_assets_root())? {
            if let Ok(e) = dir_entry {
                let p = e.path();
                let ext = p.extension().unwrap_or_else(|| std::ffi::OsStr::new(""));
                if p.is_file() && ext != "map" {
                    let content = {
                        if ext == "html" || ext == "js" || ext == "css" {
                            std::fs::read_to_string(&p)?
                        } else {
                            base64::encode(&std::fs::read(&p)?)
                        }
                    };

                    assets.insert(
                        p.strip_prefix(canister_info.get_output_assets_root())
                            .expect("Cannot strip prefix.")
                            .to_str()
                            .expect("Could not get path.")
                            .to_string(),
                        content,
                    );
                }
            }
        }

        match build_file(env, &config, &name, &assets) {
            Ok(_) => {}
            Err(e) => {
                build_stage_bar
                    .finish_with_message(&format!(r#"Failed to build canister "{}":"#, name));
                return Err(e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::env::temp_dir;
    use std::fs;
    use std::io::{Read, Write};
    use std::path::PathBuf;
    use std::process;

    #[test]
    /// Runs "echo" instead of the compiler to make sure the binaries are called in order
    /// with the good arguments.
    fn build_file_wasm() {
        // Create a binary cache environment that just returns "echo", so we can test the STDOUT.
        struct TestEnv<'a> {
            out_file: &'a fs::File,
        }

        impl<'a> BinaryResolverEnv for TestEnv<'a> {
            fn get_binary_command_path(&self, binary_name: &str) -> DfxResult<PathBuf> {
                // We need to implement this function as it's used to set the "MOC_RTS"
                // environment variable and pass the stdlib package. For the
                // purposes of this test we just return the name of the binary
                // that was requested.
                Ok(PathBuf::from(binary_name))
            }
            fn get_binary_command(&self, binary_name: &str) -> DfxResult<process::Command> {
                let stdout = self.out_file.try_clone()?;
                let stderr = self.out_file.try_clone()?;

                let mut cmd = process::Command::new("echo");

                cmd.arg(binary_name)
                    .stdout(process::Stdio::from(stdout))
                    .stderr(process::Stdio::from(stderr));

                Ok(cmd)
            }
        }

        let temp_path = temp_dir().join("stdout").with_extension("txt");
        let mut out_file = fs::File::create(temp_path.clone()).expect("Could not create file.");
        let env = TestEnv {
            out_file: &out_file,
        };

        motoko_compile(
            &env,
            None,
            "",
            Path::new("/in/file.mo"),
            Path::new("/out/file.wasm"),
            &HashMap::new(),
        )
        .expect("Function failed (motoko_compile)");
        didl_compile(&env, Path::new("/in/file.mo"), Path::new("/out/file.did"))
            .expect("Function failed (didl_compile)");
        build_did_js(
            &env,
            Path::new("/out/file.did"),
            Path::new("/out/file.did.js"),
        )
        .expect("Function failed (build_did_js)");

        out_file.flush().expect("Could not flush.");

        let mut s = String::new();
        fs::File::open(temp_path)
            .and_then(|mut f| f.read_to_string(&mut s))
            .expect("Could not read temp file.");

        assert_eq!(
            s.trim(),
            r#"moc -c /dev/stdin --debug -o /out/file.wasm --package stdlib stdlib
                moc --idl /in/file.mo -o /out/file.did --package stdlib stdlib
                didc --js /out/file.did -o /out/file.did.js"#
                .replace("                ", "")
        );
    }

    #[test]
    /// Runs "echo" instead of the compiler to make sure the binaries are called in order
    /// with the good arguments.
    fn build_file_wat() {
        // Create a binary cache environment that just returns "echo", so we can test the STDOUT.
        struct TestEnv {}

        impl BinaryResolverEnv for TestEnv {
            fn get_binary_command_path(&self, _binary_name: &str) -> DfxResult<PathBuf> {
                panic!("get_binary_command_path should not be called.")
            }
            fn get_binary_command(&self, _binary_name: &str) -> DfxResult<process::Command> {
                panic!("get_binary_command should not be called.")
            }
        }

        let env = TestEnv {};
        let wat = r#"(module )"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.into_path();
        let input_path = temp_path.join("input.wat");
        let output_path = temp_path.join("out/name/input.wasm");

        assert!(!output_path.exists());

        std::fs::write(input_path.as_path(), wat).expect("Could not create input.");
        let config = Config::from_str_and_path(
            temp_path.join("dfx.json"),
            r#"
            {
                "canisters": {
                    "name": {
                        "main": "input.wat"
                    }
                },
                "defaults": {
                    "build": {
                        "output": "out/"
                    }
                }
            }
        "#,
        )
        .unwrap();

        build_file(&env, &config, "name", &HashMap::new()).expect("Function failed - build_file");
        assert!(output_path.exists());
    }
}
