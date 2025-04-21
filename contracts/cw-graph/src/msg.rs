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
        fid: String,
        value: Option<String>,
    },
    DeleteCyberlink {
        fid: String,
    },
    UpdateAdmins {
        new_admins: Vec<String>
    },
    UpdateExecutors {
        new_executors: Vec<String>
    },
    CreateVertexAndLink {
        /// Data for the new vertex (node) to be created.
        vertex_type: String,
        vertex_value: Option<String>, // Optional value for the new vertex

        /// Data for the new link (edge) to be created.
        link_type: String,
        link_value: Option<String>, // Optional value for the new link

        /// Specifies the connection point for the link.
        /// Exactly ONE of these must be Some, indicating the pre-existing vertex.
        /// The other implicitly refers to the newly created vertex.
        link_from_existing_id: Option<String>, // If Some, the link goes FROM this existing vertex TO the new one.
        link_to_existing_id: Option<String>,   // If Some, the link goes FROM the new vertex TO this existing one.
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CountsResponse)]
    GetGraphStats {
        // If owner is Some, returns owner_count and owner_type_count (if type_ is also Some)
        owner: Option<String>,
        // If type_ is Some, returns type_count and owner_type_count (if owner is also Some)
        #[serde(rename = "type")]
        type_: Option<String>,
    },
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    DebugState {},
    
    // Global IDs API
    #[returns(Uint64)]
    LastGID {},
    #[returns(CyberlinkState)]
    CyberlinkByGID {
        gid: Uint64,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByGIDs {
        start_after_gid: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksSetByGIDs {
        gids: Vec<u64>,
    },

    // Formatted IDs API (default IDs)
    #[returns(CyberlinkState)]
    CyberlinkByFID {
        fid: String,
    },
    #[returns(Vec<(String, CyberlinkState)>)]
    CyberlinksByFIDs {
        start_after_fid: Option<String>,
        limit: Option<u32>,
    },
    #[returns(Vec<(String, CyberlinkState)>)]
    CyberlinksSetByFIDs {
        fids: Vec<String>,
    },

    // Formatted IDs API (WIP)
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByType {
        #[serde(rename = "type")]
        type_: String,
        start_after_gid: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByFrom {
        from: String,
        start_after_gid: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByTo {
        to: String,
        start_after_gid: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByOwner {
        owner: String,
        start_after_gid: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByOwnerTime {
        owner: String,
        start_time: Timestamp,
        end_time: Option<Timestamp>,
        start_after_gid: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByOwnerTimeAny {
        owner: String,
        start_time: Timestamp,
        end_time: Option<Timestamp>,
        start_after_gid: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Vec<(u64, CyberlinkState)>)]
    CyberlinksByOwnerAndType {
        owner: String,
        #[serde(rename = "type")]
        type_: String,
        start_after_gid: Option<u64>,
        limit: Option<u32>,
    },
}

// Response struct for count queries
#[cw_serde]
pub struct CountsResponse {
    pub owner_count: Option<Uint64>,
    pub type_count: Option<Uint64>,
    pub owner_type_count: Option<Uint64>,
}
