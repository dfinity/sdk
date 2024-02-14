use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::{error_invalid_argument, error_invalid_data, error_unknown};
use anyhow::{bail, Context};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use bytes::Bytes;
use candid::types::{value::IDLValue, Function, Type, TypeEnv, TypeInner};
use candid::{Decode, Encode, IDLArgs, Principal};
use candid_parser::error::pretty_diagnose;
use candid_parser::utils::CandidSource;
use dfx_core::fs::create_dir_all;
use fn_error_context::context;
use num_traits::FromPrimitive;
use reqwest::{Client, StatusCode, Url};
use rust_decimal::Decimal;
use socket2::{Domain, Socket};
use std::collections::BTreeMap;
use std::io::{stderr, stdin, stdout, IsTerminal, Read};
use std::net::{IpAddr, SocketAddr, TcpListener};
use std::path::Path;
use std::time::Duration;

pub mod assets;
pub mod clap;
pub mod currency_conversion;
pub mod stderr_wrapper;

const DECIMAL_POINT: char = '.';

// The user can pass in port "0" to dfx start i.e. "127.0.0.1:0" or "[::1]:0",
// thus, we need to recreate SocketAddr with the kernel-provided dynamically allocated port here.
// TcpBuilder is used with reuse_address and reuse_port set to "true" because
// the Actix HttpServer in webserver.rs will bind to this SocketAddr.
#[context("Failed to find reusable socket address")]
pub fn get_reusable_socket_addr(ip: IpAddr, port: u16) -> DfxResult<SocketAddr> {
    let socket = if ip.is_ipv4() {
        Socket::new(Domain::IPV4, socket2::Type::STREAM, None)
            .context("Failed to create IPv4 socket.")?
    } else {
        Socket::new(Domain::IPV6, socket2::Type::STREAM, None)
            .context("Failed to create IPv6 socket.")?
    };
    socket
        .set_reuse_address(true)
        .context("Failed to set option reuse_address of tcp builder.")?;
    // On Windows, SO_REUSEADDR without SO_EXCLUSIVEADDRUSE acts like SO_REUSEPORT (among other things), so this is only necessary on *nix.
    #[cfg(unix)]
    socket
        .set_reuse_port(true)
        .context("Failed to set option reuse_port of tcp builder.")?;
    socket
        .set_linger(Some(Duration::from_secs(10)))
        .context("Failed to set linger duration of tcp listener.")?;
    socket
        .bind(&SocketAddr::new(ip, port).into())
        .with_context(|| format!("Failed to bind socket to {}:{}.", ip, port))?;
    socket.listen(128).context("Failed to listen on socket.")?;

    let listener: TcpListener = socket.into();
    listener
        .local_addr()
        .context("Failed to fetch local address.")
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
                .read_state_canister_metadata(canister_id, metadata)
                .await
                .ok()?,
        )
        .into(),
    )
}

pub async fn fetch_remote_did_file(
    agent: &ic_agent::Agent,
    canister_id: Principal,
) -> Option<String> {
    Some(
        match read_module_metadata(agent, canister_id, "candid:service").await {
            Some(candid) => candid,
            None => {
                let bytes = agent
                    .query(&canister_id, "__get_candid_interface_tmp_hack")
                    .with_arg(Encode!().ok()?)
                    .call()
                    .await
                    .ok()?;
                Decode!(&bytes, String).ok()?
            }
        },
    )
}

/// Parse IDL file into TypeEnv. This is a best effort function: it will succeed if
/// the IDL file can be parsed and type checked in Rust parser, and has an
/// actor in the IDL file. If anything fails, it returns None.
pub fn get_candid_type(candid: CandidSource, method_name: &str) -> Option<(TypeEnv, Function)> {
    let (env, ty) = candid.load().ok()?;
    let actor = ty?;
    let method = env.get_method(&actor, method_name).ok()?.clone();
    Some((env, method))
}

pub fn get_candid_init_type(idl_path: &std::path::Path) -> Option<(TypeEnv, Function)> {
    let (env, ty) = CandidSource::File(idl_path).load().ok()?;
    let actor = ty?;
    let args = match actor.as_ref() {
        TypeInner::Class(args, _) => args.clone(),
        _ => vec![],
    };
    let res = Function {
        args,
        rets: vec![],
        modes: vec![],
    };
    Some((env, res))
}

pub fn arguments_from_file(file_name: &Path) -> DfxResult<String> {
    if file_name == Path::new("-") {
        let mut content = String::new();
        stdin().read_to_string(&mut content).map_err(|e| {
            error_invalid_argument!("Could not read arguments from stdin to string: {}", e)
        })?;
        Ok(content)
    } else {
        std::fs::read_to_string(file_name)
            .map_err(|e| error_invalid_argument!("Could not read arguments file to string: {}", e))
    }
}

#[context("Failed to create argument blob.")]
pub fn blob_from_arguments(
    dfx_env: Option<&dyn Environment>,
    arguments: Option<&str>,
    random: Option<&str>,
    arg_type: Option<&str>,
    method_type: &Option<(TypeEnv, Function)>,
) -> DfxResult<Vec<u8>> {
    let arg_type = arg_type.unwrap_or("idl");
    match arg_type {
        "raw" => {
            let bytes = hex::decode(arguments.unwrap_or("")).map_err(|e| {
                error_invalid_argument!("Argument is not a valid hex string: {}", e)
            })?;
            Ok(bytes)
        }
        "idl" => {
            let typed_args = match method_type {
                None => {
                    let arguments = arguments.unwrap_or("()");
                    candid_parser::parse_idl_args(arguments)
                        .or_else(|e| pretty_wrap("Candid argument", arguments, e))
                        .map_err(|e| error_invalid_argument!("Invalid Candid values: {}", e))?
                        .to_bytes()
                }
                Some((env, func)) => {
                    let is_terminal =
                        stdin().is_terminal() && stdout().is_terminal() && stderr().is_terminal();
                    if let Some(arguments) = arguments {
                        fuzzy_parse_argument(arguments, env, &func.args)
                    } else if func.args.is_empty() {
                        use candid::Encode;
                        Encode!()
                    } else if func
                        .args
                        .iter()
                        .all(|t| matches!(t.as_ref(), TypeInner::Opt(_)))
                    {
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
                        let config = candid_parser::configs::Configs::from_dhall(random)
                            .context("Failed to create candid parser config.")?;
                        let args = candid_parser::random::any(&seed, &config, env, &func.args)
                            .context("Failed to create idl args.")?;
                        eprintln!("Sending the following random argument:\n{}\n", args);
                        args.to_bytes_with_types(env, &func.args)
                    } else if is_terminal {
                        use candid_parser::assist::{input_args, Context};
                        let mut ctx = Context::new(env.clone());
                        if let Some(env) = dfx_env {
                            let principals = gather_principals_from_env(env);
                            if !principals.is_empty() {
                                let mut map = BTreeMap::new();
                                map.insert("principal".to_string(), principals);
                                ctx.set_completion(map);
                            }
                        }
                        let args = input_args(&ctx, &func.args)?;
                        eprintln!("Sending the following argument:\n{}\n", args);
                        eprintln!("Do you want to send this message? [y/N]");
                        let mut input = String::new();
                        stdin().read_line(&mut input)?;
                        if !["y", "Y", "yes", "Yes", "YES"].contains(&input.trim()) {
                            return Err(error_invalid_data!("User cancelled."));
                        }
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

pub fn gather_principals_from_env(env: &dyn Environment) -> BTreeMap<String, String> {
    let mut res: BTreeMap<String, String> = BTreeMap::new();
    if let Ok(mgr) = env.new_identity_manager() {
        let logger = env.get_logger();
        let mut map = mgr.get_unencrypted_principal_map(logger);
        res.append(&mut map);
    }
    if let Ok(canisters) = env.get_canister_id_store() {
        let mut canisters = canisters.get_name_id_map();
        res.append(&mut canisters);
    }
    res
}

pub fn fuzzy_parse_argument(
    arg_str: &str,
    env: &TypeEnv,
    types: &[Type],
) -> Result<Vec<u8>, candid::Error> {
    let first_char = arg_str.chars().next();
    let is_candid_format = first_char.map_or(false, |c| c == '(');
    // If parsing fails and method expects a single value, try parsing as IDLValue.
    // If it still fails, and method expects a text type, send arguments as text.
    let args = candid_parser::parse_idl_args(arg_str).or_else(|_| {
        if types.len() == 1 && !is_candid_format {
            let is_quote = first_char.map_or(false, |c| c == '"');
            if &TypeInner::Text == types[0].as_ref() && !is_quote {
                Ok(IDLValue::Text(arg_str.to_string()))
            } else {
                candid_parser::parse_idl_value(arg_str)
                    .or_else(|e| pretty_wrap("Candid argument", arg_str, e))
            }
            .map(|v| IDLArgs::new(&[v]))
        } else {
            candid_parser::parse_idl_args(arg_str)
                .or_else(|e| pretty_wrap("Candid argument", arg_str, e))
        }
    });
    let bytes = args
        .map_err(|e| error_invalid_argument!("Invalid Candid values: {}", e))?
        .to_bytes_with_types(env, types)
        .map_err(|e| error_invalid_data!("Unable to serialize Candid values: {}", e))?;
    Ok(bytes)
}

fn pretty_wrap<T>(
    file_name: &str,
    source: &str,
    e: candid_parser::Error,
) -> Result<T, candid_parser::Error> {
    pretty_diagnose(file_name, source, &e)?;
    Err(e)
}

pub fn format_as_trillions(amount: u128) -> String {
    const SCALE: u32 = 12; // trillion = 10^12
    const FRACTIONAL_PRECISION: u32 = 3;

    // handling edge case when wallet has more than ~10^29 cycles:
    // ::from_u128() returns None if the value is too big to be handled by rust_decimal,
    // in such case, the integer will be simply divided by 10^(SCALE-FRACTIONAL_PRECISION)
    // and returned as int with manually inserted comma character, therefore sacrificing
    // the fractional precision rounding (which is otherwise provided by rust_decimal)
    if let Some(mut dec) = Decimal::from_u128(amount) {
        // safe to .unwrap(), because .set_scale() throws Error only when
        // precision argument is bigger than 28, in our case it's always 12
        dec.set_scale(SCALE).unwrap();
        dec.round_dp(FRACTIONAL_PRECISION).to_string()
    } else {
        let mut v = (amount / 10u128.pow(SCALE - FRACTIONAL_PRECISION)).to_string();
        v.insert(v.len() - FRACTIONAL_PRECISION as usize, DECIMAL_POINT);
        v
    }
}

pub fn pretty_thousand_separators(num: String) -> String {
    /// formats a number provided as string, by dividing digits into groups of 3 using a delimiter
    /// https://en.wikipedia.org/wiki/Decimal_separator#Digit_grouping

    // 1. walk backwards (reverse string) and return characters until decimal point is seen
    // 2. once decimal point is seen, start counting chars and:
    //   - every third character but not at the end of the string: return (char + delimiter)
    //   - otherwise: return char
    // 3. re-reverse the string
    const GROUP_DELIMITER: char = ',';
    let mut count: u32 = 0;
    let mut seen_decimal_point = false;
    num.chars()
        .rev()
        .enumerate()
        .map(|(idx, c)| {
            if c == DECIMAL_POINT {
                seen_decimal_point = true;
                count += 1;
                c.to_string()
            } else if seen_decimal_point
                && count.rem_euclid(3) == 0
                && count > 0
                && num.len() != idx + 1
            {
                count += 1;
                format!("{}{}", c, GROUP_DELIMITER)
            } else if count == 0 {
                c.to_string()
            } else {
                count += 1;
                c.to_string()
            }
        })
        .collect::<String>()
        .chars()
        .rev()
        .collect::<_>()
}

#[context("Failed to download {} to {}.", from, to.display())]
pub async fn download_file_to_path(from: &Url, to: &Path) -> DfxResult {
    let parent_dir = to.parent().unwrap();
    create_dir_all(parent_dir)?;
    let body = download_file(from).await?;
    dfx_core::fs::write(to, body)?;
    Ok(())
}

#[context("Failed to download from url: {}.", from)]
pub async fn download_file(from: &Url) -> DfxResult<Vec<u8>> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .context("Could not create HTTP client.")?;

    let mut retry_policy = ExponentialBackoff::default();

    let body = loop {
        match attempt_download(&client, from).await {
            Ok(Some(body)) => break body,
            Ok(None) => bail!("Not found: {}", from),
            Err(request_error) => match retry_policy.next_backoff() {
                Some(duration) => tokio::time::sleep(duration).await,
                None => bail!(request_error),
            },
        }
    };

    Ok(body.to_vec())
}

async fn attempt_download(client: &Client, url: &Url) -> DfxResult<Option<Bytes>> {
    let response = client.get(url.clone()).send().await?;

    if response.status() == StatusCode::NOT_FOUND {
        Ok(None)
    } else {
        let body = response.error_for_status()?.bytes().await?;
        Ok(Some(body))
    }
}

#[cfg(test)]
mod tests {
    use super::{format_as_trillions, pretty_thousand_separators};

    #[test]
    fn prettify_balance_amount() {
        // thousands separator
        assert_eq!("3.456", pretty_thousand_separators("3.456".to_string()));
        assert_eq!("33.456", pretty_thousand_separators("33.456".to_string()));
        assert_eq!("333.456", pretty_thousand_separators("333.456".to_string()));
        assert_eq!(
            "3,333.456",
            pretty_thousand_separators("3333.456".to_string())
        );
        assert_eq!(
            "13,333.456",
            pretty_thousand_separators("13333.456".to_string())
        );
        assert_eq!(
            "313,333.456",
            pretty_thousand_separators("313333.456".to_string())
        );
        assert_eq!(
            "3,313,333.456",
            pretty_thousand_separators("3313333.456".to_string())
        );

        // scaling number
        assert_eq!("0.000", format_as_trillions(0));
        assert_eq!("0.000", format_as_trillions(1234));
        assert_eq!("0.000", format_as_trillions(500000000));
        assert_eq!("0.001", format_as_trillions(500000001));
        assert_eq!("0.168", format_as_trillions(167890100000));
        assert_eq!("1.268", format_as_trillions(1267890100000));
        assert_eq!("12.568", format_as_trillions(12567890100000));
        assert_eq!("1234.568", format_as_trillions(1234567890100000));
        assert_eq!(
            "123456123412.348",
            format_as_trillions(123456123412347890100000)
        );
        assert_eq!(
            "10000000000000000.000",
            format_as_trillions(9999999999999999999999999999)
        );
        assert_eq!(
            "99999999999999999.999",
            format_as_trillions(99999999999999999999999999999)
        );
        assert_eq!(
            "340282366920938463463374607.431",
            format_as_trillions(u128::MAX)
        );

        // combined
        assert_eq!("0.000", pretty_thousand_separators(format_as_trillions(0)));
        assert_eq!(
            "100.000",
            pretty_thousand_separators(format_as_trillions(100000000000000))
        );
        assert_eq!(
            "10,000,000,000.000",
            pretty_thousand_separators(format_as_trillions(10000000000000000000000))
        );
        assert_eq!(
            "340,282,366,920,938,463,463,374,607.431",
            pretty_thousand_separators(format_as_trillions(u128::MAX))
        );
    }
}
