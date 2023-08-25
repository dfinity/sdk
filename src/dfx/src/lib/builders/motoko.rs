use crate::lib::builders::{
    BuildConfig, BuildOutput, CanisterBuilder, IdlBuildOutput, WasmBuildOutput,
};
use crate::lib::canister_info::motoko::MotokoCanisterInfo;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{BuildError, DfxError, DfxResult};
use crate::lib::metadata::names::{CANDID_ARGS, CANDID_SERVICE};
use crate::lib::models::canister::CanisterPool;
use crate::lib::package_arguments::{self, PackageArguments};
use anyhow::Context;
use candid::Principal as CanisterId;
use dfx_core::config::cache::Cache;
use dfx_core::config::model::dfinity::{MetadataVisibility, Profile};
use fn_error_context::context;
use slog::{info, o, trace, warn, Logger};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Output;
use std::sync::Arc;

pub struct MotokoBuilder {
    logger: slog::Logger,
    cache: Arc<dyn Cache>,
}

impl MotokoBuilder {
    #[context("Failed to create MotokoBuilder.")]
    pub fn new(env: &dyn Environment) -> DfxResult<Self> {
        Ok(MotokoBuilder {
            logger: env.get_logger().new(o! {
                "module" => "motoko"
            }),
            cache: env.get_cache(),
        })
    }
}

#[context("Failed to find imports for canister at '{}'.", file.to_string_lossy())]
fn get_imports(cache: &dyn Cache, file: &Path) -> DfxResult<Vec<MotokoImport>> {
    let mut command = cache.get_binary_command("moc")?;
    let command = command.arg("--print-deps").arg(file);
    let output = command
        .output()
        .with_context(|| format!("Error executing {:#?}", command))?;

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| MotokoImport::try_from(line).context("Failed to create MotokoImport."))
        .collect::<DfxResult<Vec<MotokoImport>>>()
}

impl CanisterBuilder for MotokoBuilder {
    #[context("Failed to get dependencies for canister '{}'.", info.get_name())]
    fn get_dependencies(
        &self,
        pool: &CanisterPool,
        info: &CanisterInfo,
    ) -> DfxResult<Vec<CanisterId>> {
        let mut result = BTreeSet::new();
        let motoko_info = info.as_info::<MotokoCanisterInfo>()?;

        #[context("Failed recursive dependency detection at {}.", file.to_string_lossy())]
        fn find_deps_recursive(
            cache: &dyn Cache,
            file: &Path,
            result: &mut BTreeSet<MotokoImport>,
        ) -> DfxResult {
            if result.contains(&MotokoImport::Relative(file.to_path_buf())) {
                return Ok(());
            }

            result.insert(MotokoImport::Relative(file.to_path_buf()));

            for import in get_imports(cache, file)? {
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

    #[context("Failed to build Motoko canister '{}'.", canister_info.get_name())]
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

        let id_map = pool
            .get_canister_list()
            .iter()
            .map(|c| (c.get_name().to_string(), c.canister_id().to_text()))
            .collect();

        std::fs::create_dir_all(motoko_info.get_output_root()).with_context(|| {
            format!(
                "Failed to create {}.",
                motoko_info.get_output_root().to_string_lossy()
            )
        })?;
        let cache = &self.cache;
        let idl_dir_path = &config.idl_root;
        std::fs::create_dir_all(idl_dir_path)
            .with_context(|| format!("Failed to create {}.", idl_dir_path.to_string_lossy()))?;

        let management_idl = r##"
        type canister_id = principal;
        type wasm_module = blob;

        type canister_settings = record {
          controllers : opt vec principal;
          compute_allocation : opt nat;
          memory_allocation : opt nat;
          freezing_threshold : opt nat;
        };

        type definite_canister_settings = record {
          controllers : vec principal;
          compute_allocation : nat;
          memory_allocation : nat;
          freezing_threshold : nat;
        };

        type change_origin = variant {
          from_user : record {
            user_id : principal;
          };
          from_canister : record {
            canister_id : principal;
            canister_version : opt nat64;
          };
        };

        type change_details = variant {
          creation : record {
            controllers : vec principal;
          };
          code_uninstall;
          code_deployment : record {
            mode : variant {install; reinstall; upgrade};
            module_hash : blob;
          };
          controllers_change : record {
            controllers : vec principal;
          };
        };

        type change = record {
          timestamp_nanos : nat64;
          canister_version : nat64;
          origin : change_origin;
          details : change_details;
        };

        type http_header = record { name: text; value: text };

        type http_response = record {
          status: nat;
          headers: vec http_header;
          body: blob;
        };

        type ecdsa_curve = variant { secp256k1; };

        type satoshi = nat64;

        type bitcoin_network = variant {
          mainnet;
          testnet;
        };

        type bitcoin_address = text;

        type block_hash = blob;

        type outpoint = record {
          txid : blob;
          vout : nat32
        };

        type utxo = record {
          outpoint: outpoint;
          value: satoshi;
          height: nat32;
        };

        type get_utxos_request = record {
          address : bitcoin_address;
          network: bitcoin_network;
          filter: opt variant {
            min_confirmations: nat32;
            page: blob;
          };
        };

        type get_current_fee_percentiles_request = record {
          network: bitcoin_network;
        };

        type get_utxos_response = record {
          utxos: vec utxo;
          tip_block_hash: block_hash;
          tip_height: nat32;
          next_page: opt blob;
        };

        type get_balance_request = record {
          address : bitcoin_address;
          network: bitcoin_network;
          min_confirmations: opt nat32;
        };

        type send_transaction_request = record {
          transaction: blob;
          network: bitcoin_network;
        };

        type millisatoshi_per_byte = nat64;

        service ic : {
          create_canister : (record {
            settings : opt canister_settings;
            sender_canister_version : opt nat64;
          }) -> (record {canister_id : canister_id});
          update_settings : (record {
            canister_id : principal;
            settings : canister_settings;
            sender_canister_version : opt nat64;
          }) -> ();
          install_code : (record {
            mode : variant {install; reinstall; upgrade};
            canister_id : canister_id;
            wasm_module : wasm_module;
            arg : blob;
            sender_canister_version : opt nat64;
          }) -> ();
          uninstall_code : (record {
            canister_id : canister_id;
            sender_canister_version : opt nat64;
          }) -> ();
          start_canister : (record {canister_id : canister_id}) -> ();
          stop_canister : (record {canister_id : canister_id}) -> ();
          canister_status : (record {canister_id : canister_id}) -> (record {
              status : variant { running; stopping; stopped };
              settings: definite_canister_settings;
              module_hash: opt blob;
              memory_size: nat;
              cycles: nat;
              idle_cycles_burned_per_day: nat;
          });
          canister_info : (record {
              canister_id : canister_id;
              num_requested_changes : opt nat64;
          }) -> (record {
              total_num_changes : nat64;
              recent_changes : vec change;
              module_hash : opt blob;
              controllers : vec principal;
          });
          delete_canister : (record {canister_id : canister_id}) -> ();
          deposit_cycles : (record {canister_id : canister_id}) -> ();
          raw_rand : () -> (blob);
          http_request : (record {
            url : text;
            max_response_bytes: opt nat64;
            method : variant { get; head; post };
            headers: vec http_header;
            body : opt blob;
            transform : opt record {
              function : func (record {response : http_response; context : blob}) -> (http_response) query;
              context : blob
            };
          }) -> (http_response);

          // Threshold ECDSA signature
          ecdsa_public_key : (record {
            canister_id : opt canister_id;
            derivation_path : vec blob;
            key_id : record { curve: ecdsa_curve; name: text };
          }) -> (record { public_key : blob; chain_code : blob; });
          sign_with_ecdsa : (record {
            message_hash : blob;
            derivation_path : vec blob;
            key_id : record { curve: ecdsa_curve; name: text };
          }) -> (record { signature : blob });

          // bitcoin interface
          bitcoin_get_balance: (get_balance_request) -> (satoshi);
          bitcoin_get_utxos: (get_utxos_request) -> (get_utxos_response);
          bitcoin_send_transaction: (send_transaction_request) -> ();
          bitcoin_get_current_fee_percentiles: (get_current_fee_percentiles_request) -> (vec millisatoshi_per_byte);

          // provisional interfaces for the pre-ledger world
          provisional_create_canister_with_cycles : (record {
            amount: opt nat;
            settings : opt canister_settings;
            specified_id: opt canister_id;
            sender_canister_version : opt nat64;
          }) -> (record {canister_id : canister_id});
          provisional_top_up_canister :
            (record { canister_id: canister_id; amount: nat }) -> ();
        }
        "##;

        // FIXME move the candid out to a file
        // FIXME calling get imports twice?
        if get_imports(cache.as_ref(), input_path)?
            .contains(&MotokoImport::Ic("aaaaa-aa".to_string()))
        {
            let management_idl_path = idl_dir_path.join("aaaaa-aa.did");
            std::fs::write(&management_idl_path, management_idl).with_context(|| {
                format!("Failed to write {}.", management_idl_path.to_string_lossy())
            })?;
        }

        let package_arguments =
            package_arguments::load(cache.as_ref(), motoko_info.get_packtool())?;

        let moc_arguments = match motoko_info.get_args() {
            Some(args) => [
                package_arguments,
                args.split_whitespace().map(str::to_string).collect(),
            ]
            .concat(),
            None => package_arguments,
        };

        let candid_service_metadata_visibility = canister_info
            .get_metadata(CANDID_SERVICE)
            .map(|m| m.visibility)
            .unwrap_or(MetadataVisibility::Public);

        let candid_args_metadata_visibility = canister_info
            .get_metadata(CANDID_ARGS)
            .map(|m| m.visibility)
            .unwrap_or(MetadataVisibility::Public);

        // Generate wasm
        let params = MotokoParams {
            build_target: match profile {
                Profile::Release => BuildTarget::Release,
                _ => BuildTarget::Debug,
            },
            suppress_warning: false,
            input: input_path,
            package_arguments: &moc_arguments,
            candid_service_metadata_visibility,
            candid_args_metadata_visibility,
            output: output_wasm_path,
            idl_path: idl_dir_path,
            idl_map: &id_map,
        };
        motoko_compile(&self.logger, cache.as_ref(), &params)?;

        Ok(BuildOutput {
            canister_id: canister_info
                .get_canister_id()
                .expect("Could not find canister ID."),
            wasm: WasmBuildOutput::File(motoko_info.get_output_wasm_path().to_path_buf()),
            idl: IdlBuildOutput::File(motoko_info.get_output_idl_path().to_path_buf()),
        })
    }

    fn generate_idl(
        &self,
        _pool: &CanisterPool,
        info: &CanisterInfo,
        _config: &BuildConfig,
    ) -> DfxResult<PathBuf> {
        let generate_output_dir = &info
            .get_declarations_config()
            .output
            .as_ref()
            .context("output here must not be None")?;

        std::fs::create_dir_all(generate_output_dir).with_context(|| {
            format!(
                "Failed to create {}.",
                generate_output_dir.to_string_lossy()
            )
        })?;

        let output_idl_path = generate_output_dir
            .join(info.get_name())
            .with_extension("did");

        // get the path to candid file from dfx build
        let motoko_info = info.as_info::<MotokoCanisterInfo>()?;
        let idl_from_build = motoko_info.get_output_idl_path().to_path_buf();

        dfx_core::fs::copy(&idl_from_build, &output_idl_path)?;
        dfx_core::fs::set_permissions_readwrite(&output_idl_path)?;

        Ok(output_idl_path)
    }
}

type CanisterIdMap = BTreeMap<String, String>;
enum BuildTarget {
    Release,
    Debug,
}

struct MotokoParams<'a> {
    build_target: BuildTarget,
    idl_path: &'a Path,
    idl_map: &'a CanisterIdMap,
    package_arguments: &'a PackageArguments,
    candid_service_metadata_visibility: MetadataVisibility,
    candid_args_metadata_visibility: MetadataVisibility,
    output: &'a Path,
    input: &'a Path,
    // The following fields are control flags for dfx and will not be used by self.to_args()
    suppress_warning: bool,
}

impl MotokoParams<'_> {
    fn to_args(&self, cmd: &mut std::process::Command) {
        cmd.arg(self.input);
        cmd.arg("-o").arg(self.output);
        match self.build_target {
            BuildTarget::Release => cmd.args(["-c", "--release"]),
            BuildTarget::Debug => cmd.args(["-c", "--debug"]),
        };
        cmd.arg("--idl").arg("--stable-types");
        if self.candid_service_metadata_visibility == MetadataVisibility::Public {
            // moc defaults to private metadata, if this argument is not present.
            cmd.arg("--public-metadata").arg(CANDID_SERVICE);
        }
        if self.candid_args_metadata_visibility == MetadataVisibility::Public {
            // moc defaults to private metadata, if this argument is not present.
            cmd.arg("--public-metadata").arg(CANDID_ARGS);
        }
        if !self.idl_map.is_empty() {
            cmd.arg("--actor-idl").arg(self.idl_path);
            for (name, canister_id) in self.idl_map.iter() {
                cmd.args(["--actor-alias", name, canister_id]);
            }
        };
        cmd.args(self.package_arguments);
    }
}

/// Compile a motoko file.
#[context("Failed to compile Motoko.")]
fn motoko_compile(logger: &Logger, cache: &dyn Cache, params: &MotokoParams<'_>) -> DfxResult {
    let mut cmd = cache.get_binary_command("moc")?;
    params.to_args(&mut cmd);
    run_command(logger, &mut cmd, params.suppress_warning).context("Failed to run 'moc'.")?;
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
                    return Err(DfxError::new(BuildError::DependencyError(format!(
                        "Unknown import {}",
                        line
                    ))));
                }
                let (url, fullpath) = line.split_at(index + 1);
                (url.trim_end(), Some(fullpath))
            }
            None => (line, None),
        };
        let import = match url.find(':') {
            Some(index) => {
                if index >= line.len() - 1 {
                    return Err(DfxError::new(BuildError::DependencyError(format!(
                        "Unknown import {}",
                        url
                    ))));
                }
                let (prefix, name) = url.split_at(index + 1);
                match prefix {
                    "canister:" => MotokoImport::Canister(name.to_owned()),
                    "ic:" => MotokoImport::Ic(name.to_owned()),
                    "mo:" => MotokoImport::Lib(name.to_owned()),
                    _ => {
                        return Err(DfxError::new(BuildError::DependencyError(format!(
                            "Unknown import {}",
                            url
                        ))))
                    }
                }
            }
            None => match fullpath {
                Some(fullpath) => {
                    let path = PathBuf::from(fullpath);
                    if !path.is_file() {
                        return Err(DfxError::new(BuildError::DependencyError(format!(
                            "Cannot find import file {}",
                            path.display()
                        ))));
                    };
                    MotokoImport::Relative(path)
                }
                None => {
                    return Err(DfxError::new(BuildError::DependencyError(format!(
                        "Cannot resolve relative import {}",
                        url
                    ))))
                }
            },
        };

        Ok(import)
    }
}

fn run_command(
    logger: &slog::Logger,
    cmd: &mut std::process::Command,
    suppress_warning: bool,
) -> DfxResult<Output> {
    trace!(logger, r#"Running {}..."#, format!("{:?}", cmd));

    let output = cmd.output().context("Error while executing command.")?;
    if !output.status.success() {
        Err(DfxError::new(BuildError::CommandError(
            format!("{:?}", cmd),
            output.status,
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        )))
    } else {
        if !output.stdout.is_empty() {
            info!(logger, "{}", String::from_utf8_lossy(&output.stdout));
        }
        if !suppress_warning && !output.stderr.is_empty() {
            warn!(logger, "{}", String::from_utf8_lossy(&output.stderr));
        }
        Ok(output)
    }
}
