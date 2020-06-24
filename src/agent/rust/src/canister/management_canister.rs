use crate::agent::agent_error::AgentError;
use crate::agent::response::Replied;
use crate::agent::Agent;
use crate::{Blob, CanisterAttributes, CanisterId, RequestId};
use candid::{Decode, Encode};

const MANAGEMENT_CANISTER_ID: &str = "ic:00";
const CREATE_CMD: &str = "create_canister";
const INSTALL_CMD: &str = "install_code";

#[derive(candid::CandidType, candid::Deserialize)]
struct CreateResult {
    canister_id: candid::Principal,
}

#[derive(candid::CandidType, candid::Deserialize)]
enum InstallMode {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "reinstall")]
    Reinstall,
    #[serde(rename = "upgrade")]
    Upgrade,
}

#[derive(candid::CandidType, candid::Deserialize)]
struct CanisterInstall {
    mode: InstallMode,
    canister_id: candid::Principal,
    wasm_module: Vec<u8>,
    arg: Vec<u8>,
    compute_allocation: Option<u8>,
}

pub struct ManagementCanister<'_a> {
    pub agent: &'_a Agent,
}

impl<'_a> ManagementCanister<'_a> {
    pub async fn create_canister<W: delay::Waiter>(
        &self,
        waiter: W,
    ) -> Result<CanisterId, AgentError> {
        let request_id = self
            .agent
            .call_raw(
                &CanisterId::from_text(MANAGEMENT_CANISTER_ID).unwrap(),
                CREATE_CMD,
                &Blob::empty(),
            )
            .await?;
        match self
            .agent
            .request_status_and_wait(&request_id, waiter)
            .await?
        {
            Replied::CallReplied(blob) => {
                let cid = Decode!(blob.as_slice(), CreateResult)?;
                Ok(CanisterId::from_text(cid.canister_id.to_text())?)
            }
            reply => Err(AgentError::UnexpectedReply(reply)),
        }
    }

    pub async fn install_code(
        &self,
        canister_id: &CanisterId,
        mode: &str,
        module: &Blob,
        arg: &Blob,
        attributes: &CanisterAttributes,
    ) -> Result<RequestId, AgentError> {
        let mode = match mode {
            "install" => InstallMode::Install,
            "reinstall" => InstallMode::Reinstall,
            "upgrade" => InstallMode::Upgrade,
            &_ => InstallMode::Install,
        };
        let canister_to_install = CanisterInstall {
            mode,
            canister_id: candid::Principal::from_text(canister_id.clone().to_text())?,
            wasm_module: module.clone().as_slice().to_vec(),
            arg: arg.clone().as_slice().to_vec(),
            compute_allocation: attributes.compute_allocation.map(|x| x.into()),
        };
        let bytes: Vec<u8> = candid::Encode!(&canister_to_install).unwrap();
        self.agent
            .call_raw(
                &CanisterId::from_text(MANAGEMENT_CANISTER_ID).unwrap(),
                INSTALL_CMD,
                &Blob::from(bytes),
            )
            .await
    }
}
