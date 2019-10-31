use crate::config::dfinity::Profile;
use crate::lib::env::{BinaryResolverEnv, ProjectConfigEnv};
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::message::UserMessage;
use clap::{App, Arg, ArgMatches, SubCommand};
use std::ffi::OsStr;
use std::path::Path;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about(UserMessage::BuildCanister.to_str())
        .arg(Arg::with_name("canister").help(UserMessage::CanisterName.to_str()))
}

/// Compile a motoko file.
fn motoko_compile<T: BinaryResolverEnv>(
    env: &T,
    input_path: &Path,
    output_path: &Path,
    profile: Option<Profile>,
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

fn build_user_lib<T: BinaryResolverEnv>(
    env: &T,
    input_path: &Path,
    output_path: &Path,
) -> DfxResult {
    let output = env
        .get_binary_command("didc")?
        .arg("--js")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .output()?;

    if !output.status.success() {
        Err(DfxError::BuildError(
            BuildErrorKind::UserLibGenerationError(
                String::from_utf8_lossy(&output.stdout).to_string()
                    + &String::from_utf8_lossy(&output.stderr),
            ),
        ))
    } else {
        Ok(())
    }
}

fn build_file<T>(
    env: &T,
    profile: Option<Profile>,
    input_path: &Path,
    output_path: &Path,
) -> DfxResult
where
    T: BinaryResolverEnv,
{
    let output_wasm_path = output_path.with_extension("wasm");

    match input_path.extension().and_then(OsStr::to_str) {
        // TODO(SDK-441): Revisit supporting compilation from WAT files.
        Some("wat") => {
            let wat = std::fs::read(input_path)?;
            let wasm = wabt::wat2wasm(wat)
                .map_err(|e| DfxError::BuildError(BuildErrorKind::WatCompileError(e)))?;

            std::fs::write(&output_wasm_path, wasm)?;

            Ok(())
        }
        Some("mo") => {
            let output_idl_path = output_path.with_extension("did");
            let output_js_path = output_path.with_extension("js");

            motoko_compile(env, &input_path, &output_wasm_path, profile)?;
            didl_compile(env, &input_path, &output_idl_path)?;
            build_user_lib(env, &output_idl_path, &output_js_path)?;

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

    // get_path() returns the full path of the config file. We need to get the dirname.
    let project_root = config.get_path().parent().unwrap();

    let build_root = project_root.join(
        config
            .get_config()
            .get_defaults()
            .get_build()
            .get_output("build/"),
    );

    if let Some(canisters) = &config.get_config().canisters {
        for (k, v) in canisters {
            println!("Building {}...", k);
            if let Some(path) = &v.main {
                let path = Path::new(&path);
                let input_as_path = project_root.join(path);
                let path = path.strip_prefix("src/").unwrap_or(&path);
                let output_path = build_root.join(path).with_extension("wasm");
                std::fs::create_dir_all(output_path.parent().unwrap())?;

                build_file(
                    env,
                    config.config.profile.clone(),
                    &input_as_path,
                    &output_path,
                )?;
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
    use std::io;
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
            fn get_binary_command_path(&self, binary_name: &str) -> io::Result<PathBuf> {
                // We need to implement this function as it's used to set the "MOC_RTS"
                // environment variable and pass the stdlib package. For the
                // purposes of this test we just return the name of the binary
                // that was requested.
                Ok(PathBuf::from(binary_name))
            }
            fn get_binary_command(&self, binary_name: &str) -> io::Result<process::Command> {
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

        build_file(
            &env,
            None,
            Path::new("/in/file.mo"),
            Path::new("/out/file.wasm"),
        )
        .expect("Function failed.");

        out_file.flush().expect("Could not flush.");

        let mut s = String::new();
        fs::File::open(temp_path)
            .and_then(|mut f| f.read_to_string(&mut s))
            .expect("Could not read temp file.");

        assert_eq!(
            s.trim(),
            r#"moc /in/file.mo --debug -o /out/file.wasm --package stdlib stdlib
                moc --idl /in/file.mo -o /out/file.did --package stdlib stdlib
                didc --js /out/file.did -o /out/file.js"#
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
            fn get_binary_command_path(&self, _binary_name: &str) -> io::Result<PathBuf> {
                panic!("get_binary_command_path should not be called.")
            }
            fn get_binary_command(&self, _binary_name: &str) -> io::Result<process::Command> {
                panic!("get_binary_command should not be called.")
            }
        }

        let env = TestEnv {};
        let wat = r#"(module )"#;

        let temp_dir = tempfile::tempdir().unwrap();
        let temp_path = temp_dir.into_path();
        let input_path = temp_path.join("input.wat");
        let output_path = temp_path.join("output.wasm");

        assert!(!output_path.exists());

        std::fs::write(input_path.as_path(), wat).expect("Could not create input.");
        build_file(&env, None, input_path.as_path(), output_path.as_path())
            .expect("Function failed.");

        assert!(output_path.exists());
    }
}
