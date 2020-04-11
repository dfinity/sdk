use crate::config::dfinity::ConfigDefaultsCanisterCall;
use crate::commands::canister::create_waiter;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::{DfxError, DfxResult};
use crate::lib::message::UserMessage;
use crate::util::{blob_from_arguments, load_idl_file, print_idl_blob};

use tokio::runtime::Runtime;

pub fn exec(env: &dyn Environment, args: &ConfigDefaultsCanisterCall) -> DfxResult {
    let config = env
        .get_config()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;

    let canister_name = args.canister_name.as_ref().map(|v| v.as_str()).unwrap_or("default");
    let canister_info = CanisterInfo::load(&config, canister_name)?;
    // Read the config.
    let canister_id = canister_info.get_canister_id().ok_or_else(|| {
        DfxError::CannotFindBuildOutputForCanister(canister_info.get_name().to_owned())
    })?;
    let method_name = args.method_name.as_ref().map(|v| v.as_str()).unwrap_or("main");

    let arguments: Option<&str> = args.argument.as_ref().map(|v| v.as_str());
    let arg_type: Option<&str> = args._type.as_ref().map(|v| v.as_str());

    let is_async_flag = args._async.unwrap_or(false);
    let is_query_flag = args.query.unwrap_or(false);
    let is_update_flag = args.update.unwrap_or(false);

    let idl_ast = load_idl_file(env, canister_info.get_output_idl_path());
    let is_query = if is_async_flag {
        false
    } else {
        let is_query_method =
            idl_ast.and_then(|ast| ast.get_method_type(&method_name).map(|f| f.is_query()));
        match is_query_method {
            Some(true) => !is_update_flag,
            Some(false) => {
                if is_query_flag {
                    return Err(DfxError::InvalidMethodCall(format!(
                        "{} is not a query method",
                        method_name
                    )));
                } else {
                    false
                }
            }
            None => is_query_flag,
        }
    };

    // Get the argument, get the type, convert the argument to the type and return
    // an error if any of it doesn't work.
    let arg_value = blob_from_arguments(arguments, arg_type)?;
    let client = env
        .get_agent()
        .ok_or(DfxError::CommandMustBeRunInAProject)?;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    if is_query {
        if let Some(blob) = runtime.block_on(client.query(&canister_id, method_name, &arg_value))? {
            print_idl_blob(&blob)
                .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
        }
    } else if is_async_flag {
        let request_id = runtime.block_on(client.call(&canister_id, method_name, &arg_value))?;

        eprint!("Request ID: ");
        println!("0x{}", String::from(request_id));
    } else if let Some(blob) = runtime.block_on(client.call_and_wait(
        &canister_id,
        method_name,
        &arg_value,
        create_waiter(),
    ))? {
        print_idl_blob(&blob)
            .map_err(|e| DfxError::InvalidData(format!("Invalid IDL blob: {}", e)))?;
    }

    Ok(())
}
