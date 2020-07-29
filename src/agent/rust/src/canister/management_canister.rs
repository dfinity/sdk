use crate::agent::agent_error::AgentError;
use crate::agent::response::Replied;
use crate::agent::Agent;
use crate::{Blob, CanisterAttributes, CanisterId, RequestId};
use candid::{Decode, Encode};
use std::str::FromStr;

const MANAGEMENT_CANISTER_ID: &str = "aaaaa-aa";
const CREATE_METHOD_NAME: &str = "create_canister";
const INSTALL_METHOD_NAME: &str = "install_code";

#[derive(candid::CandidType, candid::Deserialize)]
struct CreateResult {
    canister_id: candid::Principal,
}

#[derive(Clone, candid::CandidType, candid::Deserialize)]
pub enum InstallMode {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "reinstall")]
    Reinstall,
    #[serde(rename = "upgrade")]
    Upgrade,
}

impl FromStr for InstallMode {
    type Err = AgentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "install" => Ok(InstallMode::Install),
            "reinstall" => Ok(InstallMode::Reinstall),
            "upgrade" => Ok(InstallMode::Upgrade),
            &_ => Err(AgentError::InstallModeError(s.to_string())),
        }
    }
}

#[derive(candid::CandidType, candid::Deserialize)]
struct CanisterInstall {
    mode: InstallMode,
    canister_id: candid::Principal,
    wasm_module: Vec<u8>,
    arg: Vec<u8>,
    compute_allocation: Option<u8>,
}

pub struct ManagementCanister<'agent> {
    agent: &'agent Agent,
}

impl<'agent> ManagementCanister<'agent> {
    pub fn new(agent: &'agent Agent) -> Self {
        ManagementCanister { agent }
    }

    pub async fn create_canister<W: delay::Waiter>(
        &self,
        waiter: W,
    ) -> Result<CanisterId, AgentError> {
        // candid encoding of () i.e. no arguments
        let bytes: Vec<u8> = candid::Encode!(&()).unwrap();
        let request_id = self
            .agent
            .call_raw(
                &CanisterId::from_text(MANAGEMENT_CANISTER_ID).unwrap(),
                CREATE_METHOD_NAME,
                &Blob::from(bytes),
            )
            .await?;
        match self
            .agent
            .request_status_and_wait(&request_id, waiter)
            .await?
        {
            Replied::CallReplied(blob) => {
                let cid = Decode!(blob.as_slice(), CreateResult)?;
                println!("create response id {:?}", cid.canister_id.to_text());
                Ok(CanisterId::from_text(cid.canister_id.to_text())?)
            }
            reply => Err(AgentError::UnexpectedReply(reply)),
        }
    }

    pub async fn install_code<W: delay::Waiter>(
        &self,
        waiter: W,
        canister_id: &CanisterId,
        mode: InstallMode,
        module: &Blob,
        arg: &Blob,
        attributes: &CanisterAttributes,
    ) -> Result<RequestId, AgentError> {
        let canister_to_install = CanisterInstall {
            mode,
            canister_id: candid::Principal::from_text(canister_id.to_text())?,
            wasm_module: module.as_slice().to_vec(),
            arg: arg.as_slice().to_vec(),
            compute_allocation: attributes.compute_allocation.map(|x| x.into()),
        };
        let bytes: Vec<u8> = candid::Encode!(&canister_to_install).unwrap();
        let request_id = self
            .agent
            .call_raw(
                &CanisterId::from_text(MANAGEMENT_CANISTER_ID).unwrap(),
                INSTALL_METHOD_NAME,
                &Blob::from(bytes),
            )
            .await?;
        match self
            .agent
            .request_status_and_wait(&request_id, waiter)
            .await?
        {
            // Candid type returned is () so ignoring _blob on purpose
            Replied::CallReplied(_blob) => Ok(request_id),
            reply => Err(AgentError::UnexpectedReply(reply)),
        }
    }
}
