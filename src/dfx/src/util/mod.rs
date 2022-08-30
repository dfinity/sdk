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
use num_traits::FromPrimitive;
use rust_decimal::Decimal;
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
pub mod network;
pub mod stderr_wrapper;

const DECIMAL_POINT: char = '.';

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
