use std::u64::MAX;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, StdError, StdResult, Uint64, Order, Timestamp, Env};
use cw_storage_plus::Bound;
use crate::state::{CONFIG, CyberlinkState, DELETED_IDS, ID, NAMED_CYBERLINKS, cyberlinks};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use crate::ContractError;
use crate::msg::Cyberlink;

pub fn query_last_id(deps: Deps) -> StdResult<Uint64> {
    let last_id = ID.load(deps.storage)?;
    Ok(Uint64::new(last_id))
}

pub fn query_id(deps: Deps, id: Uint64) -> StdResult<CyberlinkState> {
    // Check if the cyberlink is deleted
    if DELETED_IDS.may_load(deps.storage, id.u64())?.unwrap_or(false) {
        return Err(StdError::not_found("deleted cyberlink"));
    }

    // Load the cyberlink state
    let cyberlink = cyberlinks().load(deps.storage, id.u64())?;
    Ok(cyberlink)
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
    let cyberlinks = cyberlinks()
        .range(deps.storage, None, None, Order::Ascending)
        .map(|i| i.unwrap())
        .collect::<Vec<(u64, CyberlinkState)>>();
    let named_cyberlinks = NAMED_CYBERLINKS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|i| i.unwrap())
        .collect::<Vec<(String, CyberlinkState)>>();

    Ok(StateResponse {
        cyberlinks,
        named_cyberlinks
    })
}

// settings for pagination
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 50;

pub fn query_cyberlinks(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let cyberlinks = cyberlinks()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|i| i.unwrap())
        .collect::<Vec<(u64, CyberlinkState)>>();
    Ok(cyberlinks)
}

pub fn query_cyberlinks_by_owner(deps: Deps, owner: String, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    // Use the owner index to query cyberlinks by owner
    let cyberlinks: StdResult<Vec<_>> = cyberlinks()
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

    cyberlinks
}

pub fn query_named_cyberlinks(deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<Vec<(String, CyberlinkState)>> {
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let cyberlinks = NAMED_CYBERLINKS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|i| i.unwrap())
        .collect::<Vec<(String, CyberlinkState)>>();
    Ok(cyberlinks)
}

pub fn query_cyberlinks_by_ids(deps: Deps, ids: Vec<u64>) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let mut links: Vec<(u64, CyberlinkState)> = vec![];

    for id in ids {
        // Skip deleted cyberlinks
        if DELETED_IDS.may_load(deps.storage, id)?.unwrap_or(false) {
            continue;
        }
        let cyberlink = cyberlinks().load(deps.storage, id)?;
        links.push((id, cyberlink));
    }

    Ok(links)
}

pub fn query_cyberlinks_by_owner_time(
    deps: Deps,
    env: Env,
    owner: String,
    start_time: Timestamp,
    end_time: Option<Timestamp>,
    start_after: Option<u64>,
    limit: Option<u32>
) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    // Use current block time if end_time is not provided
    let end = end_time.unwrap_or(env.block.time);
    
    // Convert Timestamp to u64 (nanos)
    let start_nanos = start_time.nanos();
    let end_nanos = end.nanos();
    
    // Query using the created_at index with bounds
    let cyberlinks = cyberlinks()
        .idx
        .created_at
        // Use sub_prefix with just the owner (first part of composite key)
        .sub_prefix(owner_addr)
        .range(
            deps.storage,
            // Use bounds on just the timestamp part
            Some(Bound::exclusive((start_nanos, start_after.unwrap_or(0u64)))),
            Some(Bound::inclusive((end_nanos, u64::MAX))),
            Order::Ascending,
        )
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;
    
    Ok(cyberlinks)
}

pub fn query_cyberlinks_by_owner_time_any(
    deps: Deps,
    env: Env,
    owner: String,
    start_time: Timestamp,
    end_time: Option<Timestamp>,
    start_after: Option<u64>,
    limit: Option<u32>
) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    // Use current block time if end_time is not provided
    let end = end_time.unwrap_or(env.block.time);
    
    // Convert Timestamp to u64 (nanos)
    let start_nanos = start_time.nanos();
    let end_nanos = end.nanos();
    
    // Get cyberlinks by created_at time
    let created_cyberlinks = cyberlinks()
        .idx
        .created_at
        .sub_prefix(owner_addr.clone())
        .range(
            deps.storage,
            Some(Bound::exclusive((start_nanos, start_after.unwrap_or(0u64)))),
            Some(Bound::inclusive((end_nanos, u64::MAX))),
            Order::Ascending,
        )
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;
    
    // Get cyberlinks by updated_at time
    let updated_cyberlinks = cyberlinks()
        .idx
        .updated_at
        .sub_prefix(owner_addr)
        .range(
            deps.storage,
            Some(Bound::exclusive((start_nanos, start_after.unwrap_or(0u64)))),
            Some(Bound::inclusive((end_nanos, u64::MAX))),
            Order::Ascending,
        )
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;
    
    // Merge and deduplicate the results
    let mut all_cyberlinks = created_cyberlinks;
    
    // Add cyberlinks from updated_at if they're not already in the list
    for (id, cyberlink) in updated_cyberlinks {
        if !all_cyberlinks.iter().any(|(existing_id, _)| *existing_id == id) {
            all_cyberlinks.push((id, cyberlink));
        }
    }
    
    // Sort by ID for consistent results
    all_cyberlinks.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Apply limit
    let result = all_cyberlinks.into_iter().take(limit).collect();
    
    Ok(result)
}

#[cw_serde]
pub struct StateResponse {
    pub cyberlinks: Vec<(u64, CyberlinkState)>,
    pub named_cyberlinks: Vec<(String, CyberlinkState)>
}