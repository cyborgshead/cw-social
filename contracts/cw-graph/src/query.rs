use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, StdError, StdResult, Uint64, Order, Timestamp, Env};
use cw_storage_plus::Bound;
use crate::state::{CONFIG, DeeplinkState, DELETED_IDS, ID, NAMED_DEEPLINKS, deeplinks};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use crate::ContractError;
use crate::msg::Deeplink;

pub fn query_last_id(deps: Deps) -> StdResult<Uint64> {
    let last_id = ID.load(deps.storage)?;
    Ok(Uint64::new(last_id))
}

pub fn query_id(deps: Deps, id: Uint64) -> StdResult<DeeplinkState> {
    // Check if the deeplink is deleted
    if DELETED_IDS.may_load(deps.storage, id.u64())?.unwrap_or(false) {
        return Err(StdError::not_found("deleted deeplink"));
    }

    // Load the deeplink state
    let deeplink = deeplinks().load(deps.storage, id.u64())?;
    Ok(deeplink)
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admins: cfg.admins.into_iter().map(|a| a.into()).collect(),
        executors: cfg.executors.into_iter().map(|a| a.into()).collect()
    })
}

#[cw_serde]
pub struct ConfigResponse {
    pub admins: Vec<String>,
    pub executors: Vec<String>,
}

pub fn query_state(deps: Deps) -> StdResult<StateResponse> {
    let deeplinks = deeplinks()
        .range(deps.storage, None, None, Order::Ascending)
        .map(|i| i.unwrap())
        .collect::<Vec<(u64, DeeplinkState)>>();
    let named_deeplinks = NAMED_DEEPLINKS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|i| i.unwrap())
        .collect::<Vec<(String, DeeplinkState)>>();

    Ok(StateResponse {
        deeplinks,
        named_deeplinks
    })
}

// settings for pagination
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 50;

pub fn query_deeplinks(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Vec<(u64, DeeplinkState)>> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let deeplinks = deeplinks()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|i| i.unwrap())
        .collect::<Vec<(u64, DeeplinkState)>>();
    Ok(deeplinks)
}

pub fn query_deeplinks_by_owner(deps: Deps, owner: String, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Vec<(u64, DeeplinkState)>> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    // Use the owner index to query deeplinks by owner
    let deeplinks: StdResult<Vec<_>> = deeplinks()
        .idx
        .owner
        .prefix(owner_addr)
        .range(
            deps.storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit)
        .collect();

    deeplinks
}

pub fn query_named_deeplinks(deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<Vec<(String, DeeplinkState)>> {
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let deeplinks = NAMED_DEEPLINKS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|i| i.unwrap())
        .collect::<Vec<(String, DeeplinkState)>>();
    Ok(deeplinks)
}

pub fn query_deeplinks_by_ids(deps: Deps, ids: Vec<u64>) -> StdResult<Vec<(u64, DeeplinkState)>> {
    let mut links: Vec<(u64, DeeplinkState)> = vec![];

    for id in ids {
        // Skip deleted deeplinks
        if DELETED_IDS.may_load(deps.storage, id)?.unwrap_or(false) {
            continue;
        }
        let deeplink = deeplinks().load(deps.storage, id)?;
        links.push((id, deeplink));
    }

    Ok(links)
}

pub fn query_deeplinks_by_owner_time(
    deps: Deps,
    env: Env,
    owner: String,
    start_time: Timestamp,
    end_time: Option<Timestamp>,
    start_after: Option<u64>,
    limit: Option<u32>
) -> StdResult<Vec<(u64, DeeplinkState)>> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    // Use current block time if end_time is not provided
    let end = end_time.unwrap_or(env.block.time);
    
    // Convert Timestamp to u64 (nanos)
    let start_nanos = start_time.nanos();
    let end_nanos = end.nanos();
    
    // First get all deeplinks by owner
    let all_owner_deeplinks: Vec<(u64, DeeplinkState)> = deeplinks()
        .idx
        .owner
        .prefix(owner_addr)
        .range(
            deps.storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit * 2) // Get more than we need to filter
        .collect::<StdResult<Vec<_>>>()?;
    
    // Then filter by timestamp
    let filtered_deeplinks = all_owner_deeplinks
        .into_iter()
        .filter(|(_, deeplink)| {
            let created_at_nanos = deeplink.created_at.nanos();
            created_at_nanos >= start_nanos && created_at_nanos <= end_nanos
        })
        .take(limit)
        .collect();
    
    Ok(filtered_deeplinks)
}

pub fn query_deeplinks_by_owner_time_any(
    deps: Deps,
    env: Env,
    owner: String,
    start_time: Timestamp,
    end_time: Option<Timestamp>,
    start_after: Option<u64>,
    limit: Option<u32>
) -> StdResult<Vec<(u64, DeeplinkState)>> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    // Use current block time if end_time is not provided
    let end = end_time.unwrap_or(env.block.time);
    
    // Convert Timestamp to u64 (nanos)
    let start_nanos = start_time.nanos();
    let end_nanos = end.nanos();
    
    // Get all deeplinks by owner
    let all_owner_deeplinks: Vec<(u64, DeeplinkState)> = deeplinks()
        .idx
        .owner
        .prefix(owner_addr)
        .range(
            deps.storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit * 2) // Get more than we need to filter
        .collect::<StdResult<Vec<_>>>()?;
    
    // Filter by creation or update time
    let filtered_deeplinks = all_owner_deeplinks
        .into_iter()
        .filter(|(_, deeplink)| {
            let created_at_nanos = deeplink.created_at.nanos();
            let updated_at_nanos = deeplink.updated_at.map_or(created_at_nanos, |t| t.nanos());
            
            // Include if either created or updated within the range
            (created_at_nanos >= start_nanos && created_at_nanos <= end_nanos) ||
            (updated_at_nanos >= start_nanos && updated_at_nanos <= end_nanos)
        })
        .take(limit)
        .collect();
    
    Ok(filtered_deeplinks)
}

#[cw_serde]
pub struct StateResponse {
    pub deeplinks: Vec<(u64, DeeplinkState)>,
    pub named_deeplinks: Vec<(String, DeeplinkState)>
}