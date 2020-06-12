use crate::config::cache::Cache;
use crate::config::dfinity::Profile;
use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, ManifestBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::motoko::MotokoCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::models::canister::CanisterPool;
use crate::lib::package_arguments::{self, PackageArguments};
use ic_agent::CanisterId;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::Arc;

pub struct MotokoBuilder {
    cache: Arc<dyn Cache>,
}

impl MotokoBuilder {
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(MotokoBuilder {
            cache: env.get_cache(),
        })
    }
}

impl CanisterBuilder for MotokoBuilder {
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        let mut result = BTreeSet::new();
        let motoko_info = info.as_info::<MotokoCanisterInfo>()?;

        fn find_deps_recursive(
            cache: &dyn Cache,
            file: &Path,
            result: &mut BTreeSet<MotokoImport>,
        ) -> DfxResult {
            if result.contains(&MotokoImport::Relative(file.to_path_buf())) {
                return Ok(());
            }

            let output = cache
                .get_binary_command("moc")?
                .arg("--print-deps")
                .arg(&file)
                .output()?;

            let output = String::from_utf8_lossy(&output.stdout);
            for line in output.lines() {
                let import = MotokoImport::try_from(line)?;
                match import {
                    MotokoImport::Canister(_) => {
                        result.insert(import);
                    }
                    MotokoImport::Relative(path) => {
                        find_deps_recursive(cache, path.as_path(), result)?;
                    }
                    MotokoImport::Lib(_) => (),
                    MotokoImport::Ic(_) => (),
                }
            }

            Ok(())
        }
        find_deps_recursive(
            self.cache.as_ref(),
            motoko_info.get_main_path(),
            &mut result,
        )?;

        Ok(result
            .iter()
            .filter_map(|import| {
                if let MotokoImport::Canister(name) = import {
                    pool.get_first_canister_with_name(name)
                } else {
                    None
                }
            })
            .map(|canister| canister.canister_id())
            .collect())
    }

    fn supports(&self, info: &CanisterInfo) -> bool {
        info.get_type() == "motoko"
    }

    fn build(
        &self,
        pool: &CanisterPool,
        canister_info: &CanisterInfo,
        config: &BuildConfig,
    ) -> DfxResult<BuildOutput> {
        let motoko_info = canister_info.as_info::<MotokoCanisterInfo>()?;
        let profile = config.profile;
        let input_path = motoko_info.get_main_path();
        let output_wasm_path = motoko_info.get_output_wasm_path();

        let id_map = BTreeMap::from_iter(
            pool.get_canister_list()
                .iter()
                .map(|c| (c.get_name().to_string(), c.canister_id().to_text())),
        );

        std::fs::create_dir_all(motoko_info.get_output_root())?;
        let cache = &self.cache;
        let idl_dir_path = &config.idl_root;
        std::fs::create_dir_all(&idl_dir_path)?;

        let package_arguments =
            package_arguments::load(cache.as_ref(), motoko_info.get_packtool())?;

        // Generate IDL
        let output_idl_path = motoko_info.get_output_idl_path();
        let params = MotokoParams {
            build_target: BuildTarget::IDL,
            surpress_warning: false,
            inject_code: false,
            verbose: false,
            input: &input_path,
            package_arguments: &package_arguments,
            output: &output_idl_path,
            idl_path: &idl_dir_path,
            idl_map: &id_map,
        };
        motoko_compile(cache.as_ref(), &params, &BTreeMap::new())?;

        // Generate JS code even if the canister doesn't have a frontend. It might still be
        // used by another canister's frontend.
        // Generate wasm
        let params = MotokoParams {
            build_target: match profile {
                Profile::Release => BuildTarget::Release,
                _ => BuildTarget::Debug,
            },
            // Surpress the warnings the second time we call moc
            surpress_warning: true,
            inject_code: true,
            verbose: false,
            input: &input_path,
            package_arguments: &package_arguments,
            output: &output_wasm_path,
            idl_path: &idl_dir_path,
            idl_map: &id_map,
        };
        motoko_compile(cache.as_ref(), &params, &BTreeMap::new())?;

        Ok(BuildOutput {
            canister_id: canister_info
                .get_canister_id()
                .expect("Could not find canister ID."),
            wasm: WasmBuildOutput::File(motoko_info.get_output_wasm_path().to_path_buf()),
            idl: IdlBuildOutput::File(motoko_info.get_output_idl_path().to_path_buf()),
            manifest: ManifestBuildOutput::File(canister_info.get_manifest_path().to_path_buf()),
        })
    }
}

type AssetMap = BTreeMap<String, String>;
type CanisterIdMap = BTreeMap<String, String>;

fn get_asset_fn(assets: &AssetMap) -> String {
    // Create the if/else series.
    let mut cases = String::new();
    assets.iter().for_each(|(filename, content)| {
        cases += format!(
            r#"case "{}" "{}";{endline}"#,
            filename,
            content
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", ""),
            endline = "\n"
        )
        .as_str();
    });

    format!(
        r#"
            public query func __dfx_asset_path(path: Text): async Text {par}
              switch path {par}
                {}
                case _ {par}assert false; ""{end}
              {end}
            {end};
        "#,
        cases,
        par = "{",
        end = "}"
    )
}

enum BuildTarget {
    Release,
    Debug,
    IDL,
}

struct MotokoParams<'a> {
    build_target: BuildTarget,
    idl_path: &'a Path,
    idl_map: &'a CanisterIdMap,
    package_arguments: &'a PackageArguments,
    output: &'a Path,
    // The following fields will not be used by self.to_args()
    // TODO move input into self.to_args once inject_code is deprecated.
    input: &'a Path,
    verbose: bool,
    surpress_warning: bool,
    inject_code: bool,
}

impl MotokoParams<'_> {
    fn to_args(&self, cmd: &mut std::process::Command) {
        cmd.arg("-o").arg(self.output);
        match self.build_target {
            BuildTarget::Release => cmd.args(&["-c", "--release"]),
            BuildTarget::Debug => cmd.args(&["-c", "--debug"]),
            BuildTarget::IDL => cmd.arg("--idl"),
        };
        if !self.idl_map.is_empty() {
            cmd.arg("--actor-idl").arg(self.idl_path);
            for (name, canister_id) in self.idl_map.iter() {
                cmd.args(&["--actor-alias", name, canister_id]);
            }
        };
        cmd.args(self.package_arguments);
    }
}

/// Compile a motoko file.
fn motoko_compile(cache: &dyn Cache, params: &MotokoParams<'_>, assets: &AssetMap) -> DfxResult {
    let mut cmd = cache.get_binary_command("moc")?;

    let mo_rts_path = cache.get_binary_command_path("mo-rts.wasm")?;
    let input_path = if params.inject_code {
        let input_path = params.input;
        let mut content = std::fs::read_to_string(input_path)?;
        // Because we don't have an AST (yet) we need to do some regex magic.
        // Find `actor {`
        // TODO: remove this once entire process once store assets is supported by the client.
        //       See https://github.com/dfinity-lab/dfinity/pull/2106 for reference.
        let re = regex::Regex::new(r"\bactor\s.*?\{")
            .map_err(|_| DfxError::Unknown("Could not create regex.".to_string()))?;
        if let Some(actor_idx) = re.find(&content) {
            let (before, after) = content.split_at(actor_idx.end());
            content = before.to_string() + get_asset_fn(assets).as_str() + after;
        }

        let input_path = input_path.with_extension("mo-assets".to_string());
        std::fs::write(&input_path, content.as_bytes())?;
        input_path
    } else {
        params.input.to_path_buf()
    };

    cmd.arg(&input_path);
    params.to_args(&mut cmd);
    let cmd = cmd.env("MOC_RTS", mo_rts_path.as_path());
    run_command(cmd, params.verbose, params.surpress_warning)?;

    if params.inject_code {
        std::fs::remove_file(input_path)?;
    }
    Ok(())
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq)]
enum MotokoImport {
    Canister(String),
    Ic(String),
    Lib(String),
    Relative(PathBuf),
}

impl TryFrom<&str> for MotokoImport {
    type Error = DfxError;

    fn try_from(line: &str) -> Result<Self, DfxError> {
        let (url, fullpath) = match line.find(' ') {
            Some(index) => {
                if index >= line.len() - 1 {
                    return Err(DfxError::BuildError(BuildErrorKind::DependencyError(
                        format!("Unknown import {}", line),
                    )));
                }
                let (url, fullpath) = line.split_at(index + 1);
                (url.trim_end(), Some(fullpath))
            }
            None => (line, None),
        };
        let import = match url.find(':') {
            Some(index) => {
                if index >= line.len() - 1 {
                    return Err(DfxError::BuildError(BuildErrorKind::DependencyError(
                        format!("Unknown import {}", url),
                    )));
                }
                let (prefix, name) = url.split_at(index + 1);
                match prefix {
                    "canister:" => MotokoImport::Canister(name.to_owned()),
                    "ic:" => MotokoImport::Ic(name.to_owned()),
                    "mo:" => MotokoImport::Lib(name.to_owned()),
                    _ => {
                        return Err(DfxError::BuildError(BuildErrorKind::DependencyError(
                            format!("Unknown import {}", url),
                        )))
                    }
                }
            }
            None => match fullpath {
                Some(fullpath) => {
                    let path = PathBuf::from(fullpath);
                    if !path.is_file() {
                        return Err(DfxError::BuildError(BuildErrorKind::DependencyError(
                            format!("Cannot find import file {}", path.display()),
                        )));
                    };
                    MotokoImport::Relative(path)
                }
                None => {
                    return Err(DfxError::BuildError(BuildErrorKind::DependencyError(
                        format!("Cannot resolve relative import {}", url),
                    )))
                }
            },
        };

        Ok(import)
    }
}

fn run_command(
    cmd: &mut std::process::Command,
    verbose: bool,
    surpress_warning: bool,
) -> DfxResult<Output> {
    if verbose {
        println!("{:?}", cmd);
    }
    let output = cmd.output()?;
    if !output.status.success() {
        Err(DfxError::BuildError(BuildErrorKind::CompilerError(
            format!("{:?}", cmd),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )))
    } else {
        if !surpress_warning && !output.stderr.is_empty() {
            // Cannot use eprintln, because it would interfere with the progress bar.
            println!("{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(output)
    }
}
