//! This module declares canister methods expected by the assets canister client.
pub mod asset_certification;
pub mod evidence;
pub mod state_machine;
pub mod types;
mod url_decode;

#[cfg(test)]
mod tests;

pub use crate::state_machine::StableState;
use crate::{
    asset_certification::types::http::{
        CallbackFunc, HttpRequest, HttpResponse, StreamingCallbackHttpResponse,
        StreamingCallbackToken,
    },
    state_machine::{AssetDetails, CertifiedTree, EncodedAsset, State},
    types::*,
};
use asset_certification::types::{certification::AssetKey, rc_bytes::RcBytes};
use candid::Principal;
use ic_cdk::api::{call::ManualReply, caller, data_certificate, set_certified_data, time, trap};
use std::cell::RefCell;

// Re-export for use in macros
#[doc(hidden)]
pub use candid::candid_method;
#[doc(hidden)]
pub use ic_cdk::query;
#[doc(hidden)]
pub use ic_cdk::update;
#[doc(hidden)]
pub use serde_bytes::ByteBuf;

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn api_version() -> u16 {
    2
}

pub fn authorize(other: Principal) {
    STATE.with(|s| s.borrow_mut().grant_permission(other, &Permission::Commit))
}

pub fn grant_permission(arg: GrantPermissionArguments) {
    STATE.with(|s| {
        s.borrow_mut()
            .grant_permission(arg.to_principal, &arg.permission)
    })
}

pub async fn validate_grant_permission(arg: GrantPermissionArguments) -> Result<String, String> {
    Ok(format!(
        "grant {} permission to principal {}",
        arg.permission, arg.to_principal
    ))
}

pub async fn deauthorize(other: Principal) {
    let check_access_result = if other == caller() {
        // this isn't "ManagePermissions" because these legacy methods only
        // deal with the Commit permission
        has_permission_or_is_controller(&Permission::Commit)
    } else {
        is_controller()
    };
    match check_access_result {
        Err(e) => trap(&e),
        Ok(_) => STATE.with(|s| s.borrow_mut().revoke_permission(other, &Permission::Commit)),
    }
}

pub async fn revoke_permission(arg: RevokePermissionArguments) {
    let check_access_result = if arg.of_principal == caller() {
        has_permission_or_is_controller(&arg.permission)
    } else {
        has_permission_or_is_controller(&Permission::ManagePermissions)
    };
    match check_access_result {
        Err(e) => trap(&e),
        Ok(_) => STATE.with(|s| {
            s.borrow_mut()
                .revoke_permission(arg.of_principal, &arg.permission)
        }),
    }
}

pub async fn validate_revoke_permission(arg: RevokePermissionArguments) -> Result<String, String> {
    Ok(format!(
        "revoke {} permission from principal {}",
        arg.permission, arg.of_principal
    ))
}

pub fn list_authorized() -> ManualReply<Vec<Principal>> {
    STATE.with(|s| ManualReply::one(s.borrow().list_permitted(&Permission::Commit)))
}

pub fn list_permitted(arg: ListPermittedArguments) -> ManualReply<Vec<Principal>> {
    STATE.with(|s| ManualReply::one(s.borrow().list_permitted(&arg.permission)))
}

pub async fn take_ownership() {
    let caller = ic_cdk::api::caller();
    STATE.with(|s| s.borrow_mut().take_ownership(caller))
}

pub async fn validate_take_ownership() -> Result<String, String> {
    Ok("revoke all permissions, then gives the caller Commit permissions".to_string())
}

pub fn retrieve(key: AssetKey) -> RcBytes {
    STATE.with(|s| match s.borrow().retrieve(&key) {
        Ok(bytes) => bytes,
        Err(msg) => trap(&msg),
    })
}

pub fn store(arg: StoreArg) {
    STATE.with(move |s| {
        if let Err(msg) = s.borrow_mut().store(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    });
}

pub fn create_batch() -> CreateBatchResponse {
    STATE.with(|s| match s.borrow_mut().create_batch(time()) {
        Ok(batch_id) => CreateBatchResponse { batch_id },
        Err(msg) => trap(&msg),
    })
}

pub fn create_chunk(arg: CreateChunkArg) -> CreateChunkResponse {
    STATE.with(|s| match s.borrow_mut().create_chunk(arg, time()) {
        Ok(chunk_id) => CreateChunkResponse { chunk_id },
        Err(msg) => trap(&msg),
    })
}

pub fn create_chunks(arg: CreateChunksArg) -> CreateChunksResponse {
    STATE.with(|s| match s.borrow_mut().create_chunks(arg, time()) {
        Ok(chunk_ids) => CreateChunksResponse { chunk_ids },
        Err(msg) => trap(&msg),
    })
}

pub fn create_asset(arg: CreateAssetArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().create_asset(arg) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    })
}

pub fn set_asset_content(arg: SetAssetContentArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().set_asset_content(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    })
}

pub fn unset_asset_content(arg: UnsetAssetContentArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().unset_asset_content(arg) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    })
}

pub fn delete_asset(arg: DeleteAssetArguments) {
    STATE.with(|s| {
        s.borrow_mut().delete_asset(arg);
        set_certified_data(&s.borrow().root_hash());
    });
}

pub fn clear() {
    STATE.with(|s| {
        s.borrow_mut().clear();
        set_certified_data(&s.borrow().root_hash());
    });
}

pub fn commit_batch(arg: CommitBatchArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().commit_batch(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    });
}

pub fn propose_commit_batch(arg: CommitBatchArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().propose_commit_batch(arg) {
            trap(&msg);
        }
    });
}

pub fn compute_evidence(arg: ComputeEvidenceArguments) -> Option<ByteBuf> {
    STATE.with(|s| match s.borrow_mut().compute_evidence(arg) {
        Err(msg) => trap(&msg),
        Ok(maybe_evidence) => maybe_evidence,
    })
}

pub fn commit_proposed_batch(arg: CommitProposedBatchArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().commit_proposed_batch(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    });
}

pub fn validate_commit_proposed_batch(arg: CommitProposedBatchArguments) -> Result<String, String> {
    STATE.with(|s| s.borrow_mut().validate_commit_proposed_batch(arg))
}

pub fn delete_batch(arg: DeleteBatchArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().delete_batch(arg) {
            trap(&msg);
        }
    });
}

pub fn get(arg: GetArg) -> EncodedAsset {
    STATE.with(|s| match s.borrow().get(arg) {
        Ok(asset) => asset,
        Err(msg) => trap(&msg),
    })
}

pub fn get_chunk(arg: GetChunkArg) -> GetChunkResponse {
    STATE.with(|s| match s.borrow().get_chunk(arg) {
        Ok(content) => GetChunkResponse { content },
        Err(msg) => trap(&msg),
    })
}

pub fn list() -> Vec<AssetDetails> {
    STATE.with(|s| s.borrow().list_assets())
}

pub fn certified_tree() -> CertifiedTree {
    let certificate = data_certificate().unwrap_or_else(|| trap("no data certificate available"));

    STATE.with(|s| s.borrow().certified_tree(&certificate))
}

pub fn http_request(req: HttpRequest) -> HttpResponse {
    let certificate = data_certificate().unwrap_or_else(|| trap("no data certificate available"));

    STATE.with(|s| {
        s.borrow().http_request(
            req,
            &certificate,
            CallbackFunc::new(ic_cdk::id(), "http_request_streaming_callback".to_string()),
        )
    })
}

pub fn http_request_streaming_callback(
    token: StreamingCallbackToken,
) -> StreamingCallbackHttpResponse {
    STATE.with(|s| {
        s.borrow()
            .http_request_streaming_callback(token)
            .unwrap_or_else(|msg| trap(&msg))
    })
}

pub fn get_asset_properties(key: AssetKey) -> AssetProperties {
    STATE.with(|s| {
        s.borrow()
            .get_asset_properties(key)
            .unwrap_or_else(|msg| trap(&msg))
    })
}

pub fn set_asset_properties(arg: SetAssetPropertiesArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().set_asset_properties(arg) {
            trap(&msg);
        }
    })
}

pub fn get_configuration() -> ConfigurationResponse {
    STATE.with(|s| s.borrow().get_configuration())
}

pub fn configure(arg: ConfigureArguments) {
    STATE.with(|s| s.borrow_mut().configure(arg))
}

pub fn validate_configure(arg: ConfigureArguments) -> Result<String, String> {
    Ok(format!("configure: {:?}", arg))
}

pub fn can(permission: Permission) -> Result<(), String> {
    STATE.with(|s| {
        s.borrow()
            .can(&caller(), &permission)
            .then_some(())
            .ok_or_else(|| format!("Caller does not have {} permission", permission))
    })
}

pub fn can_commit() -> Result<(), String> {
    can(Permission::Commit)
}

pub fn can_prepare() -> Result<(), String> {
    can(Permission::Prepare)
}

pub fn has_permission_or_is_controller(permission: &Permission) -> Result<(), String> {
    let caller = caller();
    let has_permission = STATE.with(|s| s.borrow().has_permission(&caller, permission));
    let is_controller = ic_cdk::api::is_controller(&caller);
    if has_permission || is_controller {
        Ok(())
    } else {
        Err(format!(
            "Caller does not have {} permission and is not a controller.",
            permission
        ))
    }
}

pub fn is_manager_or_controller() -> Result<(), String> {
    has_permission_or_is_controller(&Permission::ManagePermissions)
}

pub fn is_controller() -> Result<(), String> {
    let caller = caller();
    if ic_cdk::api::is_controller(&caller) {
        Ok(())
    } else {
        Err("Caller is not a controller.".to_string())
    }
}

pub fn init(args: Option<AssetCanisterArgs>) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.clear();
        s.grant_permission(caller(), &Permission::Commit);
    });

    if let Some(upgrade_arg) = args {
        let AssetCanisterArgs::Init(init_args) = upgrade_arg else {
            ic_cdk::trap("Cannot initialize the canister with an Upgrade argument. Please provide an Init argument.")
        };
        STATE.with(|s| {
            let mut state = s.borrow_mut();
            if let Some(set_permissions) = init_args.set_permissions {
                state.set_permissions(set_permissions);
            }
        });
    }
}

pub fn pre_upgrade() -> StableState {
    STATE.with(|s| s.take().into())
}

pub fn post_upgrade(stable_state: StableState, args: Option<AssetCanisterArgs>) {
    let set_permissions = args.and_then(|args| {
        let AssetCanisterArgs::Upgrade(UpgradeArgs { set_permissions }) = args else {ic_cdk::trap("Cannot upgrade the canister with an Init argument. Please provide an Upgrade argument.")};
        set_permissions
    });

    STATE.with(|s| {
        *s.borrow_mut() = State::from(stable_state);
        set_certified_data(&s.borrow().root_hash());
        if let Some(set_permissions) = set_permissions {
            s.borrow_mut().set_permissions(set_permissions);
        }
    });
}

#[macro_export]
macro_rules! export_canister_methods {
    () => {
        use $crate::can_commit as __ic_certified_assets_can_commit;
        use $crate::can_prepare as __ic_certified_assets_can_prepare;
        use $crate::is_controller as __ic_certified_assets_is_controller;
        use $crate::is_manager_or_controller as __ic_certified_assets_is_manager_or_controller;

        #[cfg(target_arch = "wasm32")]
        #[link_section = "icp:public supported_certificate_versions"]
        pub static SUPPORTED_CERTIFICATE_VERSIONS: [u8; 3] = *b"1,2";

        // Query methods
        #[$crate::query]
        #[$crate::candid_method(query)]
        pub fn api_version() -> u16 {
            $crate::api_version()
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        pub fn retrieve(
            key: $crate::asset_certification::types::certification::AssetKey,
        ) -> $crate::asset_certification::types::rc_bytes::RcBytes {
            $crate::retrieve(key)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        pub fn get(arg: $crate::types::GetArg) -> $crate::state_machine::EncodedAsset {
            $crate::get(arg)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        pub fn get_chunk(arg: $crate::types::GetChunkArg) -> $crate::types::GetChunkResponse {
            $crate::get_chunk(arg)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        pub fn list() -> Vec<$crate::state_machine::AssetDetails> {
            $crate::list()
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        pub fn certified_tree() -> $crate::state_machine::CertifiedTree {
            $crate::certified_tree()
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        pub fn http_request(
            req: $crate::asset_certification::types::http::HttpRequest,
        ) -> $crate::asset_certification::types::http::HttpResponse {
            $crate::http_request(req)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        pub fn http_request_streaming_callback(
            token: $crate::asset_certification::types::http::StreamingCallbackToken,
        ) -> $crate::asset_certification::types::http::StreamingCallbackHttpResponse {
            $crate::http_request_streaming_callback(token)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        pub fn get_asset_properties(
            key: $crate::asset_certification::types::certification::AssetKey,
        ) -> $crate::types::AssetProperties {
            $crate::get_asset_properties(key)
        }

        // Update methods
        #[$crate::update(guard = "__ic_certified_assets_is_manager_or_controller")]
        #[$crate::candid_method(update)]
        pub fn authorize(other: candid::Principal) {
            $crate::authorize(other)
        }

        #[$crate::update(guard = "__ic_certified_assets_is_manager_or_controller")]
        #[$crate::candid_method(update)]
        pub fn grant_permission(arg: $crate::types::GrantPermissionArguments) {
            $crate::grant_permission(arg)
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        pub async fn validate_grant_permission(
            arg: $crate::types::GrantPermissionArguments,
        ) -> Result<String, String> {
            $crate::validate_grant_permission(arg).await
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        pub async fn deauthorize(other: candid::Principal) {
            $crate::deauthorize(other).await
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        pub async fn revoke_permission(arg: $crate::types::RevokePermissionArguments) {
            $crate::revoke_permission(arg).await
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        pub async fn validate_revoke_permission(
            arg: $crate::types::RevokePermissionArguments,
        ) -> Result<String, String> {
            $crate::validate_revoke_permission(arg).await
        }

        #[$crate::update(manual_reply = true)]
        #[$crate::candid_method(update)]
        pub fn list_authorized() -> ic_cdk::api::call::ManualReply<Vec<candid::Principal>> {
            $crate::list_authorized()
        }

        #[$crate::update(manual_reply = true)]
        #[$crate::candid_method(update)]
        pub fn list_permitted(
            arg: $crate::types::ListPermittedArguments,
        ) -> ic_cdk::api::call::ManualReply<Vec<candid::Principal>> {
            $crate::list_permitted(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_is_controller")]
        #[$crate::candid_method(update)]
        pub async fn take_ownership() {
            $crate::take_ownership().await
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        pub async fn validate_take_ownership() -> Result<String, String> {
            $crate::validate_take_ownership().await
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn store(arg: $crate::types::StoreArg) {
            $crate::store(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        pub fn create_batch() -> $crate::types::CreateBatchResponse {
            $crate::create_batch()
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        pub fn create_chunk(
            arg: $crate::types::CreateChunkArg,
        ) -> $crate::types::CreateChunkResponse {
            $crate::create_chunk(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        pub fn create_chunks(
            arg: $crate::types::CreateChunksArg,
        ) -> $crate::types::CreateChunksResponse {
            $crate::create_chunks(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn create_asset(arg: $crate::types::CreateAssetArguments) {
            $crate::create_asset(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn set_asset_content(arg: $crate::types::SetAssetContentArguments) {
            $crate::set_asset_content(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn unset_asset_content(arg: $crate::types::UnsetAssetContentArguments) {
            $crate::unset_asset_content(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn delete_asset(arg: $crate::types::DeleteAssetArguments) {
            $crate::delete_asset(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn clear() {
            $crate::clear()
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn commit_batch(arg: $crate::types::CommitBatchArguments) {
            $crate::commit_batch(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        pub fn propose_commit_batch(arg: $crate::types::CommitBatchArguments) {
            $crate::propose_commit_batch(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        pub fn compute_evidence(
            arg: $crate::types::ComputeEvidenceArguments,
        ) -> Option<$crate::ByteBuf> {
            $crate::compute_evidence(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn commit_proposed_batch(arg: $crate::types::CommitProposedBatchArguments) {
            $crate::commit_proposed_batch(arg)
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        pub fn validate_commit_proposed_batch(
            arg: $crate::types::CommitProposedBatchArguments,
        ) -> Result<String, String> {
            $crate::validate_commit_proposed_batch(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        pub fn delete_batch(arg: $crate::types::DeleteBatchArguments) {
            $crate::delete_batch(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn set_asset_properties(arg: $crate::types::SetAssetPropertiesArguments) {
            $crate::set_asset_properties(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        pub fn get_configuration() -> $crate::types::ConfigurationResponse {
            $crate::get_configuration()
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        pub fn configure(arg: $crate::types::ConfigureArguments) {
            $crate::configure(arg)
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        pub fn validate_configure(
            arg: $crate::types::ConfigureArguments,
        ) -> Result<String, String> {
            $crate::validate_configure(arg)
        }
    };
}

#[test]
fn candid_interface_compatibility() {
    use candid_parser::utils::{service_compatible, CandidSource};
    use std::path::PathBuf;

    candid::export_service!();
    let new_interface = __export_service();

    let old_interface =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets.did");

    println!("Exported interface: {}", new_interface);

    service_compatible(
        CandidSource::Text(&new_interface),
        CandidSource::File(old_interface.as_path()),
    )
    .expect("The assets canister interface is not compatible with the assets.did file");
}
