use crate::config::dfinity::{Config, Profile};
use crate::config::dfx_version;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::env::{BinaryResolverEnv, ProjectConfigEnv};
use crate::lib::error::DfxError::BuildError;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::assets;
use clap::{App, Arg, ArgMatches, SubCommand};
use ic_http_agent::CanisterId;
use std::ffi::OsStr;
use std::io::Read;
use std::path::Path;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about(UserMessage::BuildCanister.to_str())
        .arg(Arg::with_name("canister").help(UserMessage::CanisterName.to_str()))
}

/// Compile a motoko file.
fn motoko_compile<T: BinaryResolverEnv>(
    env: &T,
    profile: Option<Profile>,
    input_path: &Path,
    output_path: &Path,
) -> DfxResult {
    // Invoke the compiler in debug (development) or release mode, based on the current profile:
    let arg_profile = match profile {
        Some(Profile::Release) => "--release",
        _ => "--debug",
    };

    let mo_rts_path = env.get_binary_command_path("mo-rts.wasm")?;
    let stdlib_path = env.get_binary_command_path("stdlib")?;

    let output = env
        .get_binary_command("moc")?
        .env("MOC_RTS", mo_rts_path.as_path())
        .arg(&input_path)
        .arg(arg_profile)
        .arg("-o")
        .arg(&output_path)
        .arg("--package")
        .arg("stdlib")
        .arg(&stdlib_path.as_path())
        .output()?;

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

fn build_file<T>(env: &T, config: &Config, name: &str) -> DfxResult
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

            motoko_compile(env, profile, &input_path, &output_wasm_path)?;
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

pub fn exec<T>(env: &T, args: &ArgMatches<'_>) -> DfxResult
where
    T: BinaryResolverEnv + ProjectConfigEnv,
{
    // Read the config.
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    // Get the canister name (if any).
    if let Some(canister_name) = args.value_of("canister") {
        println!("Building {}...", canister_name);
        build_file(env, &config, &canister_name)?;
    } else if let Some(canisters) = &config.get_config().canisters {
        for k in canisters.keys() {
            println!("Building {}...", k);
            build_file(env, &config, &k)?;
        }
    }

    // Run `npm run build` if there is a package.json. Ignore errors.
    if config.get_project_root().join("package.json").exists() {
        eprintln!("Building frontend code...");

        // Install node modules
        std::process::Command::new("npm")
            .arg("run")
            .arg("build")
            .env("DFX_VERSION", &dfx_version())
            .current_dir(config.get_project_root())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()?;
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
            Path::new("/in/file.mo"),
            Path::new("/out/file.wasm"),
        )
        .expect("Function failed.");
        didl_compile(&env, Path::new("/in/file.mo"), Path::new("/out/file.did"))
            .expect("Function failed");
        build_did_js(
            &env,
            Path::new("/out/file.did"),
            Path::new("/out/file.did.js"),
        )
        .expect("Function failed");

        out_file.flush().expect("Could not flush.");

        let mut s = String::new();
        fs::File::open(temp_path)
            .and_then(|mut f| f.read_to_string(&mut s))
            .expect("Could not read temp file.");

        assert_eq!(
            s.trim(),
            r#"moc /in/file.mo --debug -o /out/file.wasm --package stdlib stdlib
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

        build_file(&env, &config, "name").expect("Function failed - build_file");
        assert!(output_path.exists());
    }
}
