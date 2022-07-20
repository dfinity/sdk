use crate::lib::error::DfxResult;
use crate::{error_invalid_argument, error_invalid_data, error_unknown};

use anyhow::Context;
use candid::parser::typing::{pretty_check_file, TypeEnv};
use candid::types::{Function, Type};
use candid::Deserialize;
use candid::{parser::value::IDLValue, IDLArgs};
use fn_error_context::context;
use net2::TcpListenerExt;
use net2::{unix::UnixTcpBuilderExt, TcpBuilder};
use schemars::JsonSchema;
use serde::Serialize;
use std::convert::TryFrom;
use std::fmt::Display;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

pub mod assets;
pub mod clap;
pub mod currency_conversion;
pub mod stderr_wrapper;

// The user can pass in port "0" to dfx start or dfx bootstrap i.e. "127.0.0.1:0" or "[::1]:0",
// thus, we need to recreate SocketAddr with the kernel provided dynmically allocated port here.
// TcpBuilder is used with reuse_address and reuse_port set to "true" because
// the Actix HttpServer in webserver.rs will bind to this SocketAddr.
#[context("Failed to find reusable socket address")]
pub fn get_reusable_socket_addr(ip: IpAddr, port: u16) -> DfxResult<SocketAddr> {
    let tcp_builder = if ip.is_ipv4() {
        TcpBuilder::new_v4().context("Failed to create IPv4 builder.")?
    } else {
        TcpBuilder::new_v6().context("Failed to create IPv6 builder.")?
    };
    let listener = tcp_builder
        .reuse_address(true)
        .context("Failed to set option reuse_address of tcp builder.")?
        .reuse_port(true)
        .context("Failed to set option reuse_port of tcp builder.")?
        .bind(SocketAddr::new(ip, port))
        .with_context(|| format!("Failed to set socket of tcp builder to {}:{}.", ip, port))?
        .to_tcp_listener()
        .context("Failed to create TcpListener.")?;
    listener
        .set_linger(Some(Duration::from_secs(10)))
        .context("Failed to set linger duration of tcp listener.")?;
    listener
        .local_addr()
        .context("Failed to fectch local address.")
}

pub fn expiry_duration() -> Duration {
    // 5 minutes is max ingress timeout
    Duration::from_secs(60 * 5)
}

pub fn network_to_pathcompat(network_name: &str) -> String {
    network_name.replace(|c: char| !c.is_ascii_alphanumeric(), "_")
}

/// Deserialize and print return values from canister method.
#[context("Failed to deserialize idl blob: Invalid data.")]
pub fn print_idl_blob(
    blob: &[u8],
    output_type: Option<&str>,
    method_type: &Option<(TypeEnv, Function)>,
) -> DfxResult<()> {
    let output_type = output_type.unwrap_or("pp");
    match output_type {
        "raw" => {
            let hex_string = hex::encode(blob);
            println!("{}", hex_string);
        }
        "idl" | "pp" => {
            let result = match method_type {
                None => candid::IDLArgs::from_bytes(blob),
                Some((env, func)) => candid::IDLArgs::from_bytes_with_types(blob, env, &func.rets),
            };
            if result.is_err() {
                let hex_string = hex::encode(blob);
                eprintln!("Error deserializing blob 0x{}", hex_string);
            }
            if output_type == "idl" {
                println!("{:?}", result?);
            } else {
                println!("{}", result?);
            }
        }
        v => return Err(error_unknown!("Invalid output type: {}", v)),
    }
    Ok(())
}

pub async fn read_module_metadata(
    agent: &ic_agent::Agent,
    canister_id: candid::Principal,
    metadata: &str,
) -> Option<String> {
    Some(
        String::from_utf8_lossy(
            &agent
                .read_state_canister_metadata(canister_id, metadata, false)
                .await
                .ok()?,
        )
        .into(),
    )
}

/// Parse IDL file into TypeEnv. This is a best effort function: it will succeed if
/// the IDL file can be parsed and type checked in Rust parser, and has an
/// actor in the IDL file. If anything fails, it returns None.
pub fn get_candid_type(
    idl_path: &std::path::Path,
    method_name: &str,
) -> Option<(TypeEnv, Function)> {
    let (env, ty) = check_candid_file(idl_path).ok()?;
    let actor = ty?;
    let method = env.get_method(&actor, method_name).ok()?.clone();
    Some((env, method))
}

pub fn get_candid_init_type(idl_path: &std::path::Path) -> Option<(TypeEnv, Function)> {
    let (env, ty) = check_candid_file(idl_path).ok()?;
    let actor = ty?;
    let args = match actor {
        Type::Class(args, _) => args,
        _ => vec![],
    };
    let res = Function {
        args,
        rets: vec![],
        modes: vec![],
    };
    Some((env, res))
}

pub fn check_candid_file(idl_path: &std::path::Path) -> DfxResult<(TypeEnv, Option<Type>)> {
    //context macro does not work for the returned error type
    pretty_check_file(idl_path).with_context(|| {
        format!(
            "Candid file check failed for {}.",
            idl_path.to_string_lossy()
        )
    })
}

#[context("Failed to create argument blob.")]
pub fn blob_from_arguments(
    arguments: Option<&str>,
    random: Option<&str>,
    arg_type: Option<&str>,
    method_type: &Option<(TypeEnv, Function)>,
) -> DfxResult<Vec<u8>> {
    let arg_type = arg_type.unwrap_or("idl");
    match arg_type {
        "raw" => {
            let bytes = hex::decode(&arguments.unwrap_or("")).map_err(|e| {
                error_invalid_argument!("Argument is not a valid hex string: {}", e)
            })?;
            Ok(bytes)
        }
        "idl" => {
            let typed_args = match method_type {
                None => {
                    let arguments = arguments.unwrap_or("()");
                    candid::pretty_parse::<IDLArgs>("Candid argument", arguments)
                        .map_err(|e| error_invalid_argument!("Invalid Candid values: {}", e))?
                        .to_bytes()
                }
                Some((env, func)) => {
                    if let Some(arguments) = arguments {
                        let first_char = arguments.chars().next();
                        let is_candid_format = first_char.map_or(false, |c| c == '(');
                        // If parsing fails and method expects a single value, try parsing as IDLValue.
                        // If it still fails, and method expects a text type, send arguments as text.
                        let args = arguments.parse::<IDLArgs>().or_else(|_| {
                            if func.args.len() == 1 && !is_candid_format {
                                let is_quote = first_char.map_or(false, |c| c == '"');
                                if candid::types::Type::Text == func.args[0] && !is_quote {
                                    Ok(IDLValue::Text(arguments.to_string()))
                                } else {
                                    candid::pretty_parse::<IDLValue>("Candid argument", arguments)
                                }
                                .map(|v| IDLArgs::new(&[v]))
                            } else {
                                candid::pretty_parse::<IDLArgs>("Candid argument", arguments)
                            }
                        });
                        args.map_err(|e| error_invalid_argument!("Invalid Candid values: {}", e))?
                            .to_bytes_with_types(env, &func.args)
                    } else if func.args.is_empty() {
                        use candid::Encode;
                        Encode!()
                    } else if func.args.iter().all(|t| matches!(t, Type::Opt(_))) {
                        // If the user provided no arguments, and if all the expected arguments are
                        // optional, then use null values.
                        let nulls = vec![IDLValue::Null; func.args.len()];
                        let args = IDLArgs::new(&nulls);
                        args.to_bytes_with_types(env, &func.args)
                    } else if let Some(random) = random {
                        let random = if random.is_empty() {
                            eprintln!("Random schema is empty, using any random value instead.");
                            "{=}"
                        } else {
                            random
                        };
                        use rand::Rng;
                        let mut rng = rand::thread_rng();
                        let seed: Vec<u8> = (0..2048).map(|_| rng.gen::<u8>()).collect();
                        let config = candid::parser::configs::Configs::from_dhall(random)
                            .context("Failed to create candid parser config.")?;
                        let args = IDLArgs::any(&seed, &config, env, &func.args)
                            .context("Failed to create idl args.")?;
                        eprintln!("Sending the following random argument:\n{}\n", args);
                        args.to_bytes_with_types(env, &func.args)
                    } else {
                        return Err(error_invalid_data!("Expected arguments but found none."));
                    }
                }
            }
            .map_err(|e| error_invalid_data!("Unable to serialize Candid values: {}", e))?;
            Ok(typed_args)
        }
        v => Err(error_unknown!("Invalid type: {}", v)),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(untagged)]
pub enum SerdeVec<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> SerdeVec<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(t) => vec![t],
            Self::Many(ts) => ts,
        }
    }
}

impl<T> Default for SerdeVec<T> {
    fn default() -> Self {
        Self::Many(vec![])
    }
}

#[derive(Serialize, serde::Deserialize)]
#[serde(untagged)]
enum PossiblyStrInner<T> {
    NotStr(T),
    Str(String),
}

#[derive(Serialize, Deserialize, Default, Copy, Clone, Debug, JsonSchema)]
#[serde(try_from = "PossiblyStrInner<T>")]
pub struct PossiblyStr<T>(pub T)
where
    T: FromStr,
    T::Err: Display;

impl<T> TryFrom<PossiblyStrInner<T>> for PossiblyStr<T>
where
    T: FromStr,
    T::Err: Display,
{
    type Error = T::Err;
    fn try_from(inner: PossiblyStrInner<T>) -> Result<Self, Self::Error> {
        match inner {
            PossiblyStrInner::NotStr(t) => Ok(Self(t)),
            PossiblyStrInner::Str(str) => T::from_str(&str).map(Self),
        }
    }
}
