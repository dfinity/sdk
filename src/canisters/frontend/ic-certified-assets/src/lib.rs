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

pub static SUPPORTED_CERTIFICATE_VERSIONS: [u8; 3] = *b"1,2";

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

pub fn api_version() -> u16 {
    2
}

pub fn authorize(other: Principal) {
    with_state_mut(|s| s.grant_permission(other, &Permission::Commit))
}

pub fn grant_permission(arg: GrantPermissionArguments) {
    with_state_mut(|s| s.grant_permission(arg.to_principal, &arg.permission))
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
        Ok(_) => with_state_mut(|s| s.revoke_permission(other, &Permission::Commit)),
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
        Ok(_) => with_state_mut(|s| s.revoke_permission(arg.of_principal, &arg.permission)),
    }
}

pub async fn validate_revoke_permission(arg: RevokePermissionArguments) -> Result<String, String> {
    Ok(format!(
        "revoke {} permission from principal {}",
        arg.permission, arg.of_principal
    ))
}

pub fn list_authorized() -> ManualReply<Vec<Principal>> {
    with_state(|s| ManualReply::one(s.list_permitted(&Permission::Commit)))
}

pub fn list_permitted(arg: ListPermittedArguments) -> ManualReply<Vec<Principal>> {
    with_state(|s| ManualReply::one(s.list_permitted(&arg.permission)))
}

pub async fn take_ownership() {
    let caller = ic_cdk::api::caller();
    with_state_mut(|s| s.take_ownership(caller))
}

pub async fn validate_take_ownership() -> Result<String, String> {
    Ok("revoke all permissions, then gives the caller Commit permissions".to_string())
}

pub fn retrieve(key: AssetKey) -> RcBytes {
    with_state(|s| match s.retrieve(&key) {
        Ok(bytes) => bytes,
        Err(msg) => trap(&msg),
    })
}

pub fn store(arg: StoreArg) {
    with_state_mut(|s| {
        if let Err(msg) = s.store(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.root_hash());
    });
}

pub fn create_batch() -> CreateBatchResponse {
    with_state_mut(|s| match s.create_batch(time()) {
        Ok(batch_id) => CreateBatchResponse { batch_id },
        Err(msg) => trap(&msg),
    })
}

pub fn create_chunk(arg: CreateChunkArg) -> CreateChunkResponse {
    with_state_mut(|s| match s.create_chunk(arg, time()) {
        Ok(chunk_id) => CreateChunkResponse { chunk_id },
        Err(msg) => trap(&msg),
    })
}

pub fn create_chunks(arg: CreateChunksArg) -> CreateChunksResponse {
    with_state_mut(|s| match s.create_chunks(arg, time()) {
        Ok(chunk_ids) => CreateChunksResponse { chunk_ids },
        Err(msg) => trap(&msg),
    })
}

pub fn create_asset(arg: CreateAssetArguments) {
    with_state_mut(|s| {
        if let Err(msg) = s.create_asset(arg) {
            trap(&msg);
        }
        set_certified_data(&s.root_hash());
    })
}

pub fn set_asset_content(arg: SetAssetContentArguments) {
    with_state_mut(|s| {
        if let Err(msg) = s.set_asset_content(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.root_hash());
    })
}

pub fn unset_asset_content(arg: UnsetAssetContentArguments) {
    with_state_mut(|s| {
        if let Err(msg) = s.unset_asset_content(arg) {
            trap(&msg);
        }
        set_certified_data(&s.root_hash());
    })
}

pub fn delete_asset(arg: DeleteAssetArguments) {
    with_state_mut(|s| {
        s.delete_asset(arg);
        set_certified_data(&s.root_hash());
    });
}

pub fn clear() {
    with_state_mut(|s| {
        s.clear();
        set_certified_data(&s.root_hash());
    });
}

pub fn commit_batch(arg: CommitBatchArguments) {
    with_state_mut(|s| {
        if let Err(msg) = s.commit_batch(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.root_hash());
    });
}

pub fn propose_commit_batch(arg: CommitBatchArguments) {
    with_state_mut(|s| {
        if let Err(msg) = s.propose_commit_batch(arg) {
            trap(&msg);
        }
        set_certified_data(&s.root_hash());
    });
}

pub fn compute_evidence(arg: ComputeEvidenceArguments) -> Option<ByteBuf> {
    with_state_mut(|s| match s.compute_evidence(arg) {
        Err(msg) => trap(&msg),
        Ok(maybe_evidence) => maybe_evidence,
    })
}

pub fn commit_proposed_batch(arg: CommitProposedBatchArguments) {
    with_state_mut(|s| {
        if let Err(msg) = s.commit_proposed_batch(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.root_hash());
    });
}

pub fn validate_commit_proposed_batch(arg: CommitProposedBatchArguments) -> Result<String, String> {
    with_state_mut(|s| s.validate_commit_proposed_batch(arg))
}

pub fn delete_batch(arg: DeleteBatchArguments) {
    if let Err(msg) = with_state_mut(|s| s.delete_batch(arg)) {
        trap(&msg);
    }
}

pub fn get(arg: GetArg) -> EncodedAsset {
    with_state(|s| match s.get(arg) {
        Ok(asset) => asset,
        Err(msg) => trap(&msg),
    })
}

pub fn get_chunk(arg: GetChunkArg) -> GetChunkResponse {
    with_state(|s| match s.get_chunk(arg) {
        Ok(content) => GetChunkResponse { content },
        Err(msg) => trap(&msg),
    })
}

pub fn list() -> Vec<AssetDetails> {
    with_state(|s| s.list_assets())
}

pub fn certified_tree() -> CertifiedTree {
    let certificate = data_certificate().unwrap_or_else(|| trap("no data certificate available"));

    with_state(|s| s.certified_tree(&certificate))
}

pub fn http_request(req: HttpRequest) -> HttpResponse {
    let certificate = data_certificate().unwrap_or_else(|| trap("no data certificate available"));

    with_state(|s| {
        s.http_request(
            req,
            &certificate,
            CallbackFunc::new(ic_cdk::id(), "http_request_streaming_callback".to_string()),
        )
    })
}

pub fn http_request_streaming_callback(
    token: StreamingCallbackToken,
) -> StreamingCallbackHttpResponse {
    with_state(|s| {
        s.http_request_streaming_callback(token)
            .unwrap_or_else(|msg| trap(&msg))
    })
}

pub fn get_asset_properties(key: AssetKey) -> AssetProperties {
    with_state(|s| s.get_asset_properties(key).unwrap_or_else(|msg| trap(&msg)))
}

pub fn set_asset_properties(arg: SetAssetPropertiesArguments) {
    with_state_mut(|s| {
        if let Err(msg) = s.set_asset_properties(arg) {
            trap(&msg);
        }
    })
}

pub fn get_configuration() -> ConfigurationResponse {
    with_state(|s| s.get_configuration())
}

pub fn configure(arg: ConfigureArguments) {
    with_state_mut(|s| s.configure(arg))
}

pub fn validate_configure(arg: ConfigureArguments) -> Result<String, String> {
    Ok(format!("configure: {:?}", arg))
}

pub fn can(permission: Permission) -> Result<(), String> {
    with_state(|s| {
        s.can(&caller(), &permission)
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
    let has_permission = with_state(|s| s.has_permission(&caller, permission));
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
    with_state_mut(|s| {
        s.clear();
        s.grant_permission(caller(), &Permission::Commit);
    });

    if let Some(upgrade_arg) = args {
        let AssetCanisterArgs::Init(init_args) = upgrade_arg else {
            ic_cdk::trap("Cannot initialize the canister with an Upgrade argument. Please provide an Init argument.")
        };
        with_state_mut(|s| {
            if let Some(set_permissions) = init_args.set_permissions {
                s.set_permissions(set_permissions);
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

    with_state_mut(|s| {
        *s = State::from(stable_state);
        set_certified_data(&s.root_hash());
        if let Some(set_permissions) = set_permissions {
            s.set_permissions(set_permissions);
        }
    });
}

pub fn with_state_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut State) -> R,
{
    STATE.with(|s| f(&mut s.borrow_mut()))
}

pub fn with_state<F, R>(f: F) -> R
where
    F: FnOnce(&State) -> R,
{
    STATE.with(|s| f(&s.borrow()))
}

#[macro_export]
macro_rules! export_canister_methods {
    () => {
        use $crate::asset_certification;
        use $crate::state_machine;
        use $crate::types;
        use $crate::ByteBuf;

        use $crate::can_commit as __ic_certified_assets_can_commit;
        use $crate::can_prepare as __ic_certified_assets_can_prepare;
        use $crate::is_controller as __ic_certified_assets_is_controller;
        use $crate::is_manager_or_controller as __ic_certified_assets_is_manager_or_controller;

        #[cfg(target_arch = "wasm32")]
        #[link_section = "icp:public supported_certificate_versions"]
        static CERTIFICATE_VERSIONS: [u8; 3] = $crate::SUPPORTED_CERTIFICATE_VERSIONS;

        // Query methods
        #[$crate::query]
        #[$crate::candid_method(query)]
        fn api_version() -> u16 {
            $crate::api_version()
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        fn retrieve(
            key: asset_certification::types::certification::AssetKey,
        ) -> asset_certification::types::rc_bytes::RcBytes {
            $crate::retrieve(key)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        fn get(arg: types::GetArg) -> state_machine::EncodedAsset {
            $crate::get(arg)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        fn get_chunk(arg: types::GetChunkArg) -> types::GetChunkResponse {
            $crate::get_chunk(arg)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        fn list() -> Vec<state_machine::AssetDetails> {
            $crate::list()
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        fn certified_tree() -> state_machine::CertifiedTree {
            $crate::certified_tree()
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        fn http_request(
            req: asset_certification::types::http::HttpRequest,
        ) -> asset_certification::types::http::HttpResponse {
            $crate::http_request(req)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        fn http_request_streaming_callback(
            token: asset_certification::types::http::StreamingCallbackToken,
        ) -> asset_certification::types::http::StreamingCallbackHttpResponse {
            $crate::http_request_streaming_callback(token)
        }

        #[$crate::query]
        #[$crate::candid_method(query)]
        fn get_asset_properties(
            key: asset_certification::types::certification::AssetKey,
        ) -> types::AssetProperties {
            $crate::get_asset_properties(key)
        }

        // Update methods
        #[$crate::update(guard = "__ic_certified_assets_is_manager_or_controller")]
        #[$crate::candid_method(update)]
        fn authorize(other: candid::Principal) {
            $crate::authorize(other)
        }

        #[$crate::update(guard = "__ic_certified_assets_is_manager_or_controller")]
        #[$crate::candid_method(update)]
        fn grant_permission(arg: types::GrantPermissionArguments) {
            $crate::grant_permission(arg)
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        async fn validate_grant_permission(
            arg: types::GrantPermissionArguments,
        ) -> Result<String, String> {
            $crate::validate_grant_permission(arg).await
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        async fn deauthorize(other: candid::Principal) {
            $crate::deauthorize(other).await
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        async fn revoke_permission(arg: types::RevokePermissionArguments) {
            $crate::revoke_permission(arg).await
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        async fn validate_revoke_permission(
            arg: types::RevokePermissionArguments,
        ) -> Result<String, String> {
            $crate::validate_revoke_permission(arg).await
        }

        #[$crate::update(manual_reply = true)]
        #[$crate::candid_method(update)]
        fn list_authorized() -> ic_cdk::api::call::ManualReply<Vec<candid::Principal>> {
            $crate::list_authorized()
        }

        #[$crate::update(manual_reply = true)]
        #[$crate::candid_method(update)]
        fn list_permitted(
            arg: types::ListPermittedArguments,
        ) -> ic_cdk::api::call::ManualReply<Vec<candid::Principal>> {
            $crate::list_permitted(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_is_controller")]
        #[$crate::candid_method(update)]
        async fn take_ownership() {
            $crate::take_ownership().await
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        async fn validate_take_ownership() -> Result<String, String> {
            $crate::validate_take_ownership().await
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn store(arg: types::StoreArg) {
            $crate::store(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        fn create_batch() -> types::CreateBatchResponse {
            $crate::create_batch()
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        fn create_chunk(arg: types::CreateChunkArg) -> types::CreateChunkResponse {
            $crate::create_chunk(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        fn create_chunks(arg: types::CreateChunksArg) -> types::CreateChunksResponse {
            $crate::create_chunks(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn create_asset(arg: types::CreateAssetArguments) {
            $crate::create_asset(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn set_asset_content(arg: types::SetAssetContentArguments) {
            $crate::set_asset_content(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn unset_asset_content(arg: types::UnsetAssetContentArguments) {
            $crate::unset_asset_content(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn delete_asset(arg: types::DeleteAssetArguments) {
            $crate::delete_asset(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn clear() {
            $crate::clear()
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn commit_batch(arg: types::CommitBatchArguments) {
            $crate::commit_batch(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        fn propose_commit_batch(arg: types::CommitBatchArguments) {
            $crate::propose_commit_batch(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        fn compute_evidence(arg: types::ComputeEvidenceArguments) -> Option<ByteBuf> {
            $crate::compute_evidence(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn commit_proposed_batch(arg: types::CommitProposedBatchArguments) {
            $crate::commit_proposed_batch(arg)
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        fn validate_commit_proposed_batch(
            arg: types::CommitProposedBatchArguments,
        ) -> Result<String, String> {
            $crate::validate_commit_proposed_batch(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        fn delete_batch(arg: types::DeleteBatchArguments) {
            $crate::delete_batch(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn set_asset_properties(arg: types::SetAssetPropertiesArguments) {
            $crate::set_asset_properties(arg)
        }

        #[$crate::update(guard = "__ic_certified_assets_can_prepare")]
        #[$crate::candid_method(update)]
        fn get_configuration() -> types::ConfigurationResponse {
            $crate::get_configuration()
        }

        #[$crate::update(guard = "__ic_certified_assets_can_commit")]
        #[$crate::candid_method(update)]
        fn configure(arg: types::ConfigureArguments) {
            $crate::configure(arg)
        }

        #[$crate::update]
        #[$crate::candid_method(update)]
        fn validate_configure(arg: types::ConfigureArguments) -> Result<String, String> {
            $crate::validate_configure(arg)
        }
    };
}

#[test]
fn candid_interface_compatibility() {
    use candid_parser::utils::{service_compatible, CandidSource};
    use std::path::PathBuf;

    export_canister_methods!();

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
