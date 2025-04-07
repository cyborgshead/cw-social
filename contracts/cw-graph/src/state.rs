use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

#[cw_serde]
pub struct CyberlinkState {
    #[serde(rename = "type")]
    pub type_: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub owner: Addr,
    pub created_at: Timestamp,
    pub updated_at: Option<Timestamp>,
    pub formatted_id: Option<String>,
}

// Define the primary key namespace
pub const CYBERLINKS_KEY: &str = "cyberlinks";

// Define indexes for the cyberlinks
pub struct CyberlinkIndices<'a> {
    // Index by owner
    pub owner: MultiIndex<'a, Addr, CyberlinkState, u64>,
    // Index by type
    pub type_: MultiIndex<'a, String, CyberlinkState, u64>,
    // Index by from
    pub from: MultiIndex<'a, String, CyberlinkState, u64>,
    // Index by to
    pub to: MultiIndex<'a, String, CyberlinkState, u64>,
    
    pub created_at: MultiIndex<'a, (Addr, u64), CyberlinkState, u64>,
    pub updated_at: MultiIndex<'a, (Addr, u64), CyberlinkState, u64>,
    // Index by formatted_id
    pub formatted_id: MultiIndex<'a, String, CyberlinkState, u64>,
}

// Implement IndexList for CyberlinkIndices
impl<'a> IndexList<CyberlinkState> for CyberlinkIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<CyberlinkState>> + '_> {
        let v: Vec<&dyn Index<CyberlinkState>> = vec![
            &self.owner, &self.type_, &self.from, &self.to, 
            &self.created_at, &self.updated_at, &self.formatted_id
        ];
        Box::new(v.into_iter())
    }
}

// Create a function to get the indexed map
pub fn cyberlinks<'a>() -> IndexedMap<u64, CyberlinkState, CyberlinkIndices<'a>> {
    let indices = CyberlinkIndices {
        owner: MultiIndex::new(
            |_pk, d: &CyberlinkState| d.owner.clone(),
            CYBERLINKS_KEY,
            "cyberlinks__owner",
        ),
        type_: MultiIndex::new(
            |_pk, d: &CyberlinkState| d.type_.clone(),
            CYBERLINKS_KEY,
            "cyberlinks__type",
        ),
        from: MultiIndex::new(
            |_pk, d: &CyberlinkState| d.from.clone(),
            CYBERLINKS_KEY,
            "cyberlinks__from",
        ),
        to: MultiIndex::new(
            |_pk, d: &CyberlinkState| d.to.clone(),
            CYBERLINKS_KEY,
            "cyberlinks__to",
        ),
        
        created_at: MultiIndex::new(
            |_pk, d: &CyberlinkState| (d.owner.clone(), d.created_at.nanos()),
            CYBERLINKS_KEY,
            "cyberlinks__created_at",
        ),
        updated_at: MultiIndex::new(
            |_pk, d: &CyberlinkState| (d.owner.clone(), d.updated_at.map_or(d.created_at.nanos(), |t| t.nanos())),
            CYBERLINKS_KEY,
            "cyberlinks__updated_at",
        ),
        formatted_id: MultiIndex::new(
            |_pk, d: &CyberlinkState| d.formatted_id.clone().unwrap_or_default(),
            CYBERLINKS_KEY,
            "cyberlinks__formatted_id",
        ),
    };
    IndexedMap::new(CYBERLINKS_KEY, indices)
}

// Named cyberlinks
pub const NAMED_CYBERLINKS_KEY: &str = "named_cyberlinks";
pub const NAMED_CYBERLINKS: Map<&str, u64> = Map::new(NAMED_CYBERLINKS_KEY);

// ID counter
pub const ID_KEY: &str = "id";
pub const ID: Item<u64> = Item::new(ID_KEY);

// Type-specific ID counters
pub const TYPE_ID_KEY: &str = "type_id";
pub const TYPE_IDS: Map<&str, u64> = Map::new(TYPE_ID_KEY);

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

