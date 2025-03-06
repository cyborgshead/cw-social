use cosmwasm_schema::cw_serde;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map, MultiIndex, IndexList, IndexedMap, Index};

#[cw_serde]
pub struct DeeplinkState {
    #[serde(rename = "type")]
    pub type_: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub owner: Addr,
    pub created_at: Timestamp,
    pub updated_at: Option<Timestamp>,
}

// Define the primary key namespace
pub const DEEPLINKS_KEY: &str = "deeplinks";

// Define indexes for the deeplinks
pub struct DeeplinkIndices<'a> {
    // Index by owner
    pub owner: MultiIndex<'a, Addr, DeeplinkState, u64>,
    // Index by type
    pub type_: MultiIndex<'a, String, DeeplinkState, u64>,
    // Index by from
    pub from: MultiIndex<'a, String, DeeplinkState, u64>,
    // Index by to
    pub to: MultiIndex<'a, String, DeeplinkState, u64>,
    
    pub created_at: MultiIndex<'a, (Addr, u64), DeeplinkState, u64>,
    pub updated_at: MultiIndex<'a, (Addr, u64), DeeplinkState, u64>,
}

// Implement IndexList for DeeplinkIndices
impl<'a> IndexList<DeeplinkState> for DeeplinkIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<DeeplinkState>> + '_> {
        let v: Vec<&dyn Index<DeeplinkState>> = vec![
            &self.owner, &self.type_, &self.from, &self.to, 
            &self.created_at, &self.updated_at
        ];
        Box::new(v.into_iter())
    }
}

// Create a function to get the indexed map
pub fn deeplinks<'a>() -> IndexedMap<u64, DeeplinkState, DeeplinkIndices<'a>> {
    let indices = DeeplinkIndices {
        owner: MultiIndex::new(
            |_pk, d: &DeeplinkState| d.owner.clone(),
            DEEPLINKS_KEY,
            "deeplinks__owner",
        ),
        type_: MultiIndex::new(
            |_pk, d: &DeeplinkState| d.type_.clone(),
            DEEPLINKS_KEY,
            "deeplinks__type",
        ),
        from: MultiIndex::new(
            |_pk, d: &DeeplinkState| d.from.clone(),
            DEEPLINKS_KEY,
            "deeplinks__from",
        ),
        to: MultiIndex::new(
            |_pk, d: &DeeplinkState| d.to.clone(),
            DEEPLINKS_KEY,
            "deeplinks__to",
        ),
        
        created_at: MultiIndex::new(
            |_pk, d: &DeeplinkState| (d.owner.clone(), d.created_at.nanos()),
            DEEPLINKS_KEY,
            "deeplinks__created_at",
        ),
        updated_at: MultiIndex::new(
            |_pk, d: &DeeplinkState| (d.owner.clone(), d.updated_at.map_or(d.created_at.nanos(), |t| t.nanos())),
            DEEPLINKS_KEY,
            "deeplinks__updated_at",
        ),
    };
    IndexedMap::new(DEEPLINKS_KEY, indices)
}

// Named deeplinks
pub const NAMED_DEEPLINKS_KEY: &str = "named_deeplinks";
pub const NAMED_DEEPLINKS: Map<&str, DeeplinkState> = Map::new(NAMED_DEEPLINKS_KEY);

// ID counter
pub const ID_KEY: &str = "id";
pub const ID: Item<u64> = Item::new(ID_KEY);

// Deleted IDs tracking
pub const DELETED_IDS_KEY: &str = "deleted_ids";
pub const DELETED_IDS: Map<u64, bool> = Map::new(DELETED_IDS_KEY);

#[cw_serde]
pub struct Config {
    pub admins: Vec<Addr>,
    pub executors: Vec<Addr>
}

impl Config {
    pub fn is_admin(&self, addr: impl AsRef<str>) -> bool {
        let addr = addr.as_ref();
        self.admins.iter().any(|a| a.as_ref() == addr)
    }

    pub fn is_executor(&self, addr: impl AsRef<str>) -> bool {
        let addr = addr.as_ref();
        self.executors.iter().any(|a| a.as_ref() == addr)
    }

    pub fn can_modify(&self, addr: &str) -> bool {
        self.is_admin(addr)
    }

    pub fn can_execute(&self, addr: &str) -> bool {
        self.is_admin(addr) || self.is_executor(addr)
    }
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

