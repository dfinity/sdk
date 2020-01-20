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
use ic_http_agent::CanisterId;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Output;

type AssetMap = HashMap<String, String>;
type CanisterIdMap = HashMap<String, String>;
type CanisterDependencyMap = HashMap<String, HashSet<String>>;

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

/// Compile a motoko file.
#[allow(clippy::too_many_arguments)]
fn motoko_compile(
    cache: &dyn Cache,
    profile: Option<Profile>,
    content: &str,
    input_path: &Path,
    output_path: &Path,
    idl_path: &Path,
    id_map: &CanisterIdMap,
    assets: &AssetMap,
) -> DfxResult {
    // Invoke the compiler in debug (development) or release mode, based on the current profile:
    let arg_profile = match profile {
        Some(Profile::Release) => "--release",
        _ => "--debug",
    };

    let mo_rts_path = cache.get_binary_command_path("mo-rts.wasm")?;
    let stdlib_path = cache.get_binary_command_path("stdlib")?;

    let mut content = content.to_string();
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

    let mut rng = thread_rng();
    let input_path = input_path.with_extension(format!("mo-{}", rng.gen::<u64>()));
    std::fs::write(&input_path, content.as_bytes())?;

    let mut alias = Vec::new();
    for (name, canister_id) in id_map.iter() {
        alias.push("--actor-alias");
        alias.push(name);
        alias.push(canister_id);
    }

    let mut cmd = cache.get_binary_command("moc")?;
    let cmd = cmd
        .env("MOC_RTS", mo_rts_path.as_path())
        .arg("-c")
        .arg(&input_path)
        .arg(arg_profile)
        .arg("-o")
        .arg(&output_path)
        .arg("--package")
        .arg("stdlib")
        .arg(&stdlib_path.as_path())
        .arg("--actor-idl")
        .arg(&idl_path)
        .args(&alias);
    run_command(cmd)?;

    std::fs::remove_file(input_path)?;
    Ok(())
}

#[derive(Debug, PartialEq, Hash, Eq)]
enum MotokoImport {
    Canister(String),
    Local(PathBuf),
}

struct MotokoImports(HashSet<MotokoImport>);

impl MotokoImports {
    pub fn get_canisters(&self) -> HashSet<String> {
        let mut res = HashSet::new();
        for dep in self.0.iter() {
            if let MotokoImport::Canister(ref name) = dep {
                res.insert(name.to_owned());
            }
        }
        res
    }
}

fn find_deps(cache: &dyn Cache, input_path: &Path, deps: &mut MotokoImports) -> DfxResult {
    let import = MotokoImport::Local(input_path.to_path_buf());
    if deps.0.contains(&import) {
        return Ok(());
    }
    deps.0.insert(import);

    let mut cmd = cache.get_binary_command("moc")?;
    let cmd = cmd.arg("--print-deps").arg(&input_path);
    let output = run_command(cmd)?;

    let output = String::from_utf8_lossy(&output.stdout);
    for dep in output.lines() {
        let prefix: Vec<_> = dep.split(':').collect();
        match prefix[0] {
            "canister" => {
                deps.0.insert(MotokoImport::Canister(prefix[1].to_string()));
            }
            "ic" => (),
            // TODO resolve mo URL
            "mo" => (),
            file => {
                let path = input_path
                    .parent()
                    .unwrap()
                    .join(file)
                    .canonicalize()
                    .unwrap();
                find_deps(cache, &path, deps)?;
            }
        }
    }
    Ok(())
}

fn didl_compile(
    cache: &dyn Cache,
    input_path: &Path,
    output_path: &Path,
    idl_path: &Path,
    id_map: &CanisterIdMap,
) -> DfxResult {
    let stdlib_path = cache.get_binary_command_path("stdlib")?;

    let mut alias = Vec::new();
    for (name, canister_id) in id_map.iter() {
        alias.push("--actor-alias");
        alias.push(name);
        alias.push(canister_id);
    }

    let mut cmd = cache.get_binary_command("moc")?;
    let cmd = cmd
        .arg("--idl")
        .arg(&input_path)
        .arg("-o")
        .arg(&output_path)
        .arg("--package")
        .arg("stdlib")
        .arg(&stdlib_path.as_path())
        .arg("--actor-idl")
        .arg(&idl_path)
        .args(&alias);
    run_command(cmd)?;
    Ok(())
}

fn build_did_js(cache: &dyn Cache, input_path: &Path, output_path: &Path) -> DfxResult {
    let mut cmd = cache.get_binary_command("didc")?;
    let cmd = cmd.arg("--js").arg(&input_path).arg("-o").arg(&output_path);
    run_command(cmd)?;
    Ok(())
}

fn run_command(cmd: &mut std::process::Command) -> DfxResult<Output> {
    let output = cmd.output()?;
    if !output.status.success() {
        Err(DfxError::BuildError(BuildErrorKind::CompilerError(
            format!("{:?}", cmd),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )))
    } else {
        Ok(output)
    }
}

fn build_canister_js(canister_id: &CanisterId, canister_info: &CanisterInfo) -> DfxResult {
    let output_canister_js_path = canister_info.get_output_canister_js_path();

    let mut language_bindings = assets::language_bindings()?;

    let mut file = language_bindings.entries()?.next().unwrap()?;
    let mut file_contents = String::new();
    file.read_to_string(&mut file_contents)?;

    let new_file_contents = file_contents
        .replace("{canister_id}", &canister_id.to_text())
        .replace("{project_name}", canister_info.get_name());

    let output_canister_js_path_str = output_canister_js_path.to_str().ok_or_else(|| {
        DfxError::BuildError(BuildErrorKind::CanisterJsGenerationError(format!(
            "Unable to convert output canister js path to a string: {:#?}",
            output_canister_js_path
        )))
    })?;
    std::fs::write(output_canister_js_path_str, new_file_contents)?;

    Ok(())
}

fn build_file(
    env: &dyn Environment,
    config: &Config,
    name: &str,
    id_map: &CanisterIdMap,
    assets: &AssetMap,
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
    let idl_path = canister_info
        .get_output_root()
        .parent()
        .unwrap()
        .join("idl");
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
            let canister_id = canister_info
                .get_canister_id()
                .ok_or_else(|| DfxError::BuildError(BuildErrorKind::CouldNotReadCanisterId()))?;

            let output_did_js_path = canister_info.get_output_did_js_path();

            let output_idl_path = idl_path
                .join(canister_id.to_text().split_off(3))
                .with_extension("did");

            std::fs::create_dir_all(canister_info.get_output_root())?;

            let content = std::fs::read_to_string(input_path)?;
            let cache = env.get_cache();
            motoko_compile(
                cache.as_ref(),
                profile,
                &content,
                &input_path,
                &output_wasm_path,
                &idl_path,
                &id_map,
                assets,
            )?;
            didl_compile(
                cache.as_ref(),
                &input_path,
                &output_idl_path,
                &idl_path,
                id_map,
            )?;
            didl_compile(
                cache.as_ref(),
                &input_path,
                &canister_info.get_output_idl_path(),
                &idl_path,
                id_map,
            )?;
            build_did_js(cache.as_ref(), &output_idl_path, &output_did_js_path)?;
            build_canister_js(&canister_id, &canister_info)?;

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
    pub canisters: Vec<String>,
    seen: HashSet<String>,
    deps: CanisterDependencyMap,
}

impl BuildSequence {
    pub fn new(deps: CanisterDependencyMap) -> Self {
        let mut res = BuildSequence {
            canisters: Vec::new(),
            seen: HashSet::new(),
            deps,
        };
        res.build_dependency();
        res
    }
    fn build_dependency(&mut self) {
        let names: Vec<_> = self.deps.keys().cloned().collect();
        for name in names {
            self.dfs(&name);
        }
    }
    fn dfs(&mut self, canister: &str) {
        if self.seen.contains(canister) {
            return;
        }
        self.seen.insert(canister.to_string());
        let deps = self.deps.get(canister).unwrap().clone();
        for dep in deps {
            self.dfs(&dep);
        }
        self.canisters.push(canister.to_string());
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
    let mut id_map = HashMap::new();
    let mut deps = HashMap::new();
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
        let mut canister_deps = MotokoImports(HashSet::new());
        find_deps(env.get_cache().as_ref(), &input_path, &mut canister_deps)?;
        deps.insert(name.to_owned(), canister_deps.get_canisters());
    }

    // Sort dependency
    status_bar.set_message("Analyzing build dependency...");
    let seq = BuildSequence::new(deps);
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
        build_stage_bar.println(&format!("Building canister {}", name));
        match build_file(env, &config, name, &id_map, &HashMap::new()) {
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
        return Ok(());
    }

    build_stage_bar.println("Building frontend");

    let mut process = std::process::Command::new("npm")
        .arg("run")
        .arg("build")
        .env("DFX_VERSION", &format!("{}", dfx_version()))
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

    build_stage_bar.inc(1);

    build_stage_bar.println("Bundling frontend assets in the canister");

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

        match build_file(env, &config, &name, &id_map, &assets) {
            Ok(()) => {}
            Err(e) => {
                build_stage_bar
                    .finish_with_message(&format!(r#"Failed to build canister "{}":"#, name));
                return Err(e);
            }
        }
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

        motoko_compile(
            &cache,
            None,
            "",
            &input_path,
            Path::new("/out/file.wasm"),
            Path::new("."),
            &HashMap::new(),
            &HashMap::new(),
        )
        .expect("Function failed.");
        didl_compile(
            &cache,
            Path::new("/in/file.mo"),
            Path::new("/out/file.did"),
            Path::new("."),
            &HashMap::new(),
        )
        .expect("Function failed (didl_compile)");
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
            &r"moc -c .*?.mo-[0-9]+ --debug -o /out/file.wasm --package stdlib stdlib --actor-idl .
                moc --idl /in/file.mo -o /out/file.did --package stdlib stdlib --actor-idl .
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

        build_file(&env, &config, "name", &HashMap::new(), &HashMap::new())
            .expect("Function failed - build_file");
        assert!(output_path.exists());
    }
}
