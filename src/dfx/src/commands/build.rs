use crate::config::cache::Cache;
use crate::config::dfinity::{Config, Profile};
use crate::config::dfx_version;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxError::BuildError;
use crate::lib::error::{BuildErrorKind, DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::assets;
use clap::{App, Arg, ArgMatches, SubCommand};
use console::Style;
use ic_http_agent::CanisterId;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Output;

type AssetMap = BTreeMap<String, String>;
type CanisterIdMap = BTreeMap<String, String>;
type CanisterDependencyMap = BTreeMap<String, BTreeSet<String>>;

pub fn construct() -> App<'static, 'static> {
    SubCommand::with_name("build")
        .about(UserMessage::BuildCanister.to_str())
        .arg(
            Arg::with_name("skip-frontend")
                .long("skip-frontend")
                .takes_value(false)
                .help(UserMessage::SkipFrontend.to_str()),
        )
}

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
    }
}

/// Compile a motoko file.
fn motoko_compile(cache: &dyn Cache, params: &MotokoParams<'_>, assets: &AssetMap) -> DfxResult {
    let mut cmd = cache.get_binary_command("moc")?;

    let mo_rts_path = cache.get_binary_command_path("mo-rts.wasm")?;
    let stdlib_path = cache.get_binary_command_path("stdlib")?;
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
    let cmd = cmd
        .env("MOC_RTS", mo_rts_path.as_path())
        // TODO Move packages flags into params.to_args once dfx supports custom packages
        .arg("--package")
        .arg("stdlib")
        .arg(&stdlib_path.as_path());
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

struct MotokoImports(BTreeSet<MotokoImport>);

impl MotokoImports {
    pub fn get_canisters(&self) -> BTreeSet<String> {
        let mut res = BTreeSet::new();
        for dep in self.0.iter() {
            if let MotokoImport::Canister(ref name) = dep {
                res.insert(name.to_owned());
            }
        }
        res
    }
}

fn parse_moc_deps(line: &str) -> DfxResult<MotokoImport> {
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

fn find_deps(cache: &dyn Cache, input_path: &Path, deps: &mut MotokoImports) -> DfxResult {
    let import = MotokoImport::Relative(input_path.to_path_buf());
    if deps.0.contains(&import) {
        return Ok(());
    }
    deps.0.insert(import);

    let mut cmd = cache.get_binary_command("moc")?;
    let cmd = cmd.arg("--print-deps").arg(&input_path);
    let output = run_command(cmd, false, false)?;

    let output = String::from_utf8_lossy(&output.stdout);
    for dep in output.lines() {
        let import = parse_moc_deps(dep)?;
        match import {
            MotokoImport::Canister(_) => {
                deps.0.insert(import);
            }
            MotokoImport::Relative(path) => find_deps(cache, &path, deps)?,
            MotokoImport::Lib(_) => (),
            MotokoImport::Ic(_) => (),
        }
    }
    Ok(())
}

fn build_did_js(cache: &dyn Cache, input_path: &Path, output_path: &Path) -> DfxResult {
    let mut cmd = cache.get_binary_command("didc")?;
    let cmd = cmd.arg("--js").arg(&input_path).arg("-o").arg(&output_path);
    run_command(cmd, false, false)?;
    Ok(())
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

fn decode_path_to_str(path: &Path) -> DfxResult<&str> {
    path.to_str().ok_or_else(|| {
        DfxError::BuildError(BuildErrorKind::CanisterJsGenerationError(format!(
            "Unable to convert output canister js path to a string: {:#?}",
            path
        )))
    })
}

fn build_canister_js(canister_id: &CanisterId, canister_info: &CanisterInfo) -> DfxResult {
    let output_canister_js_path = canister_info.get_output_canister_js_path();

    let mut language_bindings = assets::language_bindings()?;

    for f in language_bindings.entries()? {
        let mut file = f?;
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents)?;

        let new_file_contents = file_contents
            .replace("{canister_id}", &canister_id.to_text())
            .replace("{project_name}", canister_info.get_name());

        match decode_path_to_str(&file.path()?)? {
            "canister.js" => {
                std::fs::write(
                    decode_path_to_str(output_canister_js_path)?,
                    new_file_contents,
                )?;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}

fn build_file(
    env: &dyn Environment,
    config: &Config,
    name: &str,
    id_map: &CanisterIdMap,
    assets: &mut AssetMap,
) -> DfxResult {
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

            Ok(())
        }

        Some("mo") => {
            std::fs::create_dir_all(canister_info.get_output_root())?;
            let cache = env.get_cache();
            let idl_dir_path = canister_info.get_idl_dir_path();
            std::fs::create_dir_all(&idl_dir_path)?;

            // Generate IDL
            let output_idl_path = canister_info.get_output_idl_path();
            let idl_file_path = canister_info
                .get_idl_file_path()
                .ok_or_else(|| DfxError::BuildError(BuildErrorKind::CouldNotReadCanisterId()))?;
            let params = MotokoParams {
                build_target: BuildTarget::IDL,
                surpress_warning: false,
                inject_code: false,
                verbose: false,
                input: &input_path,
                output: &output_idl_path,
                idl_path: &idl_dir_path,
                idl_map: &id_map,
            };
            motoko_compile(cache.as_ref(), &params, &BTreeMap::new())?;
            std::fs::copy(&output_idl_path, &idl_file_path)?;
            // Generate JS code even if the canister doesn't have a frontend. It might still be
            // used by another canister's frontend.
            let output_did_js_path = canister_info.get_output_did_js_path();
            let canister_id = canister_info
                .get_canister_id()
                .ok_or_else(|| DfxError::BuildError(BuildErrorKind::CouldNotReadCanisterId()))?;
            build_did_js(cache.as_ref(), &output_idl_path, &output_did_js_path)?;
            build_canister_js(&canister_id, &canister_info)?;
            // Add Candid and JS binding to assets.
            let candid_content = base64::encode(&std::fs::read(&output_idl_path)?);
            assets.insert("candid.did".to_owned(), candid_content);
            let did_js_content = base64::encode(&std::fs::read(&output_did_js_path)?);
            assets.insert("candid.js".to_owned(), did_js_content);
            // Generate wasm
            let params = MotokoParams {
                build_target: match profile {
                    Some(Profile::Release) => BuildTarget::Release,
                    _ => BuildTarget::Debug,
                },
                // Surpress the warnings the second time we call moc
                surpress_warning: true,
                inject_code: true,
                verbose: false,
                input: &input_path,
                output: &output_wasm_path,
                idl_path: &idl_dir_path,
                idl_map: &id_map,
            };
            motoko_compile(cache.as_ref(), &params, assets)?;

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

struct BuildSequence {
    canisters: Vec<String>,
    seen: BTreeSet<String>,
    deps: CanisterDependencyMap,
    id_map: CanisterIdMap,
}

impl BuildSequence {
    fn new(deps: CanisterDependencyMap, id_map: CanisterIdMap) -> DfxResult<Self> {
        let mut res = BuildSequence {
            canisters: Vec::new(),
            seen: BTreeSet::new(),
            deps,
            id_map,
        };
        res.build_dependency()?;
        Ok(res)
    }
    fn get_ids(&self, name: &str) -> CanisterIdMap {
        let mut res = BTreeMap::new();
        // It's okay to unwrap because we have already traversed the dependency graph without errors.
        let deps = self.deps.get(name).unwrap();
        for canister in deps.iter() {
            let id = self.id_map.get(canister).unwrap();
            res.insert(canister.to_owned(), id.to_owned());
        }
        res
    }
    fn build_dependency(&mut self) -> DfxResult {
        let names: Vec<_> = self.deps.keys().cloned().collect();
        for name in names {
            self.dfs(&name)?;
        }
        Ok(())
    }
    fn dfs(&mut self, canister: &str) -> DfxResult {
        if self.seen.contains(canister) {
            return Ok(());
        }
        self.seen.insert(canister.to_string());
        let deps = self
            .deps
            .get(canister)
            .ok_or_else(|| {
                DfxError::BuildError(BuildErrorKind::DependencyError(format!(
                    "Cannot find canister {}",
                    canister
                )))
            })?
            .clone();
        for dep in deps {
            self.dfs(&dep)?;
        }
        self.canisters.push(canister.to_string());
        Ok(())
    }
}

pub fn exec(env: &dyn Environment, args: &ArgMatches<'_>) -> DfxResult {
    // Read the config.
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    // Check the cache. This will only install the cache if there isn't one installed
    // already.
    env.get_cache().install()?;

    let green = Style::new().green().bold();

    let status_bar = ProgressBar::new_spinner();
    status_bar.set_draw_target(ProgressDrawTarget::stderr());
    status_bar.set_message("Building canisters...");
    status_bar.enable_steady_tick(80);

    let maybe_canisters = &config.get_config().canisters;
    if maybe_canisters.is_none() {
        status_bar.finish_with_message("No canisters, nothing to build.");
        return Ok(());
    }
    let canisters = maybe_canisters.as_ref().unwrap();

    // Build canister id map and dependency graph
    let mut id_map = BTreeMap::new();
    let mut deps = BTreeMap::new();
    for name in canisters.keys() {
        let canister_info = CanisterInfo::load(&config, name)?;
        status_bar.set_message("Generating canister ids...");
        // Write the CID.
        std::fs::create_dir_all(
            canister_info
                .get_canister_id_path()
                .parent()
                .expect("Cannot use root."),
        )?;
        std::fs::write(
            canister_info.get_canister_id_path(),
            canister_info.generate_canister_id()?.into_blob().0,
        )
        .map_err(DfxError::from)?;

        let canister_id = canister_info.get_canister_id().unwrap().to_text();
        id_map.insert(name.to_owned(), canister_id);

        let input_path = canister_info.get_main_path();
        let mut canister_deps = MotokoImports(BTreeSet::new());
        find_deps(env.get_cache().as_ref(), &input_path, &mut canister_deps)?;
        deps.insert(name.to_owned(), canister_deps.get_canisters());
    }

    // Sort dependency
    status_bar.set_message("Analyzing build dependency...");
    let seq = BuildSequence::new(deps, id_map)?;
    status_bar.finish_and_clear();

    let num_stages = seq.canisters.len() as u64 + 2;
    let build_stage_bar = ProgressBar::new(num_stages);
    build_stage_bar.set_draw_target(ProgressDrawTarget::stderr());
    build_stage_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{wide_bar}] {pos}/{len}")
            .progress_chars("=> "),
    );
    build_stage_bar.enable_steady_tick(80);

    // Build canister
    for name in &seq.canisters {
        build_stage_bar.println(&format!("{} canister {}", green.apply_to("Building"), name));
        match build_file(env, &config, name, &seq.get_ids(name), &mut BTreeMap::new()) {
            Ok(()) => {}
            Err(e) => {
                build_stage_bar.abandon();
                return Err(e);
            }
        }
        build_stage_bar.inc(1);
    }

    // If there is not a package.json, we don't have a frontend and can quit early.
    if !config.get_project_root().join("package.json").exists() || args.is_present("skip-frontend")
    {
        build_stage_bar.finish_and_clear();
        return Ok(());
    }

    build_stage_bar.println(&format!("{} frontend", green.apply_to("Building")));

    let mut cmd = std::process::Command::new("npm");
    cmd.arg("run")
        .arg("build")
        .env("DFX_VERSION", &format!("{}", dfx_version()))
        .current_dir(config.get_project_root())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    run_command(&mut cmd, false, false)?;

    build_stage_bar.inc(1);
    build_stage_bar.println(&format!(
        "{} frontend assets in the canister",
        green.apply_to("Bundling")
    ));

    let frontends: Vec<String> = canisters
        .iter()
        .filter(|(_, v)| v.frontend.is_some())
        .map(|(k, _)| k)
        .cloned()
        .collect();
    for name in frontends {
        let canister_info = CanisterInfo::load(&config, name.as_str()).map_err(|_| {
            BuildError(BuildErrorKind::CanisterNameIsNotInConfigError(
                name.to_owned(),
            ))
        })?;

        let mut assets: AssetMap = AssetMap::new();
        for dir_entry in std::fs::read_dir(canister_info.get_output_assets_root())? {
            if let Ok(e) = dir_entry {
                let p = e.path();
                let ext = p.extension().unwrap_or_else(|| std::ffi::OsStr::new(""));
                if p.is_file() && ext != "map" {
                    let content = base64::encode(&std::fs::read(&p)?);
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

        match build_file(env, &config, &name, &seq.get_ids(&name), &mut assets) {
            Ok(()) => {}
            Err(e) => {
                build_stage_bar.abandon();
                return Err(e);
            }
        }
    }
    // Remove generated IDL files
    // We don't want to simply remove the whole directory, as in the future,
    // we may want to keep the IDL files downloaded from network.
    for name in &seq.canisters {
        let canister_info = CanisterInfo::load(&config, name)?;
        let idl_file_path = canister_info
            .get_idl_file_path()
            .ok_or_else(|| DfxError::BuildError(BuildErrorKind::CouldNotReadCanisterId()))?;
        std::fs::remove_file(idl_file_path)?;
    }

    build_stage_bar.finish_and_clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::config::cache::MockCache;
    use crate::lib::environment::MockEnvironment;
    use std::env::temp_dir;
    use std::fs;
    use std::io::{Read, Write};
    use std::path::PathBuf;
    use std::process;
    use std::rc::Rc;

    #[test]
    /// Runs "echo" instead of the compiler to make sure the binaries are called in order
    /// with the good arguments.
    fn build_file_wasm() {
        let temp_path = temp_dir().join("stdout").with_extension("txt");
        let mut out_file = fs::File::create(temp_path.clone()).expect("Could not create file.");
        let mut cache = MockCache::default();

        cache
            .expect_get_binary_command_path()
            .returning(|x| Ok(PathBuf::from(x)));

        cache.expect_get_binary_command().returning({
            let out_file = out_file.try_clone().unwrap();
            move |binary_name| {
                let stdout = out_file.try_clone()?;
                let stderr = out_file.try_clone()?;

                let mut cmd = process::Command::new("echo");

                cmd.arg(binary_name.to_owned())
                    .stdout(process::Stdio::from(stdout))
                    .stderr(process::Stdio::from(stderr));

                Ok(cmd)
            }
        });

        let input_path = temp_dir().join("file").with_extension("mo");
        fs::File::create(input_path.clone()).expect("Could not create file.");

        let params = MotokoParams {
            build_target: BuildTarget::Debug,
            surpress_warning: false,
            inject_code: true,
            verbose: false,
            input: &input_path,
            output: Path::new("/out/file.wasm"),
            idl_path: Path::new("."),
            idl_map: &BTreeMap::new(),
        };
        motoko_compile(&cache, &params, &BTreeMap::new()).expect("Function failed.");

        let params = MotokoParams {
            build_target: BuildTarget::IDL,
            surpress_warning: false,
            inject_code: false,
            verbose: false,
            input: Path::new("/in/file.mo"),
            output: Path::new("/out/file.did"),
            idl_path: Path::new("."),
            idl_map: &BTreeMap::new(),
        };
        motoko_compile(&cache, &params, &BTreeMap::new()).expect("Function failed (didl_compile)");
        build_did_js(
            &cache,
            Path::new("/out/file.did"),
            Path::new("/out/file.did.js"),
        )
        .expect("Function failed (build_did_js)");

        out_file.flush().expect("Could not flush.");

        let mut s = String::new();
        fs::File::open(temp_path)
            .and_then(|mut f| f.read_to_string(&mut s))
            .expect("Could not read temp file.");

        let re = regex::Regex::new(
            &r"moc .*?.mo-assets -o /out/file.wasm -c --debug --package stdlib stdlib
                moc /in/file.mo -o /out/file.did --idl --package stdlib stdlib
                didc --js /out/file.did -o /out/file.did.js"
                .replace("                ", ""),
        )
        .expect("Could not create regex.");
        assert!(re.is_match(s.trim()));
    }

    #[test]
    /// Runs "echo" instead of the compiler to make sure the binaries are called in order
    /// with the good arguments.
    fn build_file_wat() {
        let mut env = MockEnvironment::default();
        env.expect_get_cache()
            .return_once_st(move || Rc::new(MockCache::default()));

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

        build_file(
            &env,
            &config,
            "name",
            &BTreeMap::new(),
            &mut BTreeMap::new(),
        )
        .expect("Function failed - build_file");
        assert!(output_path.exists());
    }
}
