use cosmwasm_std::Uint64;
use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::query::{ConfigResponse, StateResponse};
use crate::state::DeeplinkState;

#[cw_serde]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub executers: Vec<String>,
}

#[cw_serde]
pub struct NamedDeeplink {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>
}
#[cw_serde]
pub struct  Deeplink {
    #[serde(rename = "type")]
    pub type_: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateNamedDeeplink {
        name: String,
        deeplink: Deeplink,
    },
    CreateDeeplink {
        deeplink: Deeplink,
    },
    CreateDeeplinks {
        deeplinks: Vec<Deeplink>,
    },
    UpdateDeeplink {
        id: u64,
        deeplink: Deeplink,
    },
    DeleteDeeplink {
        id: Uint64,
    },
    UpdateAdmins {
        new_admins: Vec<String>
    },
    UpdateExecutors {
        new_executors: Vec<String>
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Uint64)]
    LastId {},
    #[returns(StateResponse)]
    DebugState {},
    #[returns(DeeplinkState)]
    Deeplink {
        id: Uint64,
    },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(Vec<(u64, DeeplinkState)>)]
    Deeplinks {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(String, DeeplinkState)>)]
    NamedDeeplinks {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, DeeplinkState)>)]
    DeeplinksByIds {
        ids: Vec<u64>,
    },
    #[returns(Vec<(u64, DeeplinkState)>)]
    DeeplinksByOwner {
        owner: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}
