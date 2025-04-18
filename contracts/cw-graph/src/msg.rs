use cosmwasm_std::Uint64;
use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::query::{ConfigResponse, StateResponse};
use crate::state::CyberlinkState;
use cosmwasm_std::Timestamp;

#[cw_serde]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub executers: Vec<String>,
    pub semantic_cores: Vec<String>,
}

#[cw_serde]
pub struct NamedCyberlink {
    pub id: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>
}
#[cw_serde]
pub struct Cyberlink {
    #[serde(rename = "type")]
    pub type_: String,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateNamedCyberlink {
        name: String,
        cyberlink: Cyberlink,
    },
    CreateCyberlink {
        cyberlink: Cyberlink,
    },
    CreateCyberlinks {
        cyberlinks: Vec<Cyberlink>,
    },
    UpdateCyberlink {
        id: String,
        cyberlink: Cyberlink,
    },
    DeleteCyberlink {
        id: String,
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
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    DebugState {},
    
    // Global IDs API
    #[returns(Uint64)]
    LastId {},
    #[returns(CyberlinkState)]
    Cyberlink {
        id: Uint64,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    Cyberlinks {
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    // Formatted IDs API (default IDs)
    #[returns(Vec<(String, CyberlinkState)>)]
    NamedCyberlinks {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByIds {
        ids: Vec<u64>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByOwner {
        owner: String,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByOwnerTime {
        owner: String,
        start_time: Timestamp,
        end_time: Option<Timestamp>,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByOwnerTimeAny {
        owner: String,
        start_time: Timestamp,
        end_time: Option<Timestamp>,
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(CyberlinkState)]
    CyberlinkById {
        id: String,
    },
}
