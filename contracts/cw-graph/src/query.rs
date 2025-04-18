use crate::state::{cyberlinks, CyberlinkState, CONFIG, DELETED_IDS, ID, NAMED_CYBERLINKS, OWNER_LINK_COUNT, TYPE_LINK_COUNT, OWNER_TYPE_LINK_COUNT};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Deps, Env, Order, StdError, StdResult, Timestamp, Uint64};
use cw_storage_plus::Bound;

use crate::msg::CountsResponse;

pub fn query_last_gid(deps: Deps) -> StdResult<Uint64> {
    let last_id = ID.load(deps.storage)?;
    Ok(Uint64::new(last_id))
}

pub fn query_cyberlink_by_gid(deps: Deps, id: Uint64) -> StdResult<CyberlinkState> {
    // Check if the cyberlink is deleted
    if DELETED_IDS.has(deps.storage, id.u64()) {
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
        .collect::<Vec<(String, u64)>>();

    Ok(StateResponse {
        cyberlinks,
        named_cyberlinks
    })
}

// settings for pagination
const MAX_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 50;

pub fn query_cyberlinks_by_gids(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let cyberlinks = cyberlinks()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;
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

pub fn query_cyberlinks_by_type(deps: Deps, type_: String, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    cyberlinks()
        .idx
        .type_
        .prefix(type_)
        .range(
            deps.storage,
            start,
            None,
            Order::Ascending,
        )
        .take(limit)
        .collect()
}

pub fn query_cyberlinks_by_from(deps: Deps, from: String, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    cyberlinks()
        .idx
        .from
        .prefix(from)
        .range(
            deps.storage,
            start,
            None,
            Order::Ascending,
        )
        .take(limit)
        .collect()
}

pub fn query_cyberlinks_by_to(deps: Deps, to: String, start_after: Option<u64>, limit: Option<u32>) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    cyberlinks()
        .idx
        .to
        .prefix(to)
        .range(
            deps.storage,
            start,
            None,
            Order::Ascending,
        )
        .take(limit)
        .collect()
}

pub fn query_cyberlinks_by_ids(deps: Deps, start_after: Option<String>, limit: Option<u32>) -> StdResult<Vec<(String, CyberlinkState)>> {
    let start = start_after.map(|s| Bound::ExclusiveRaw(s.into()));
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let results = NAMED_CYBERLINKS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| -> StdResult<Option<(String, CyberlinkState)>> {
            let (formatted_id, global_id) = item?;
            if DELETED_IDS.has(deps.storage, global_id) {
                return Ok(None); // Skip deleted
            }
            match cyberlinks().may_load(deps.storage, global_id)? {
                Some(state) => Ok(Some((formatted_id, state))),
                None => Ok(None), // Skip if GID not found in cyberlinks (should be rare)
            }
        })
        .filter_map(Result::transpose) // Filter out None values and propagate Err
        .collect::<StdResult<Vec<_>>>()?;

    Ok(results)
}

pub fn query_cyberlinks_set_by_gids(deps: Deps, ids: Vec<u64>) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let mut links: Vec<(u64, CyberlinkState)> = vec![];

    for id in ids {
        // Skip deleted cyberlinks
        if DELETED_IDS.has(deps.storage, id) {
            continue;
        }
        // Use may_load to handle non-existent IDs gracefully
        match cyberlinks().may_load(deps.storage, id) {
            Ok(Some(cyberlink)) => links.push((id, cyberlink)),
            Ok(None) => {} // GID not found, skip
            Err(e) => return Err(e), // Propagate other errors
        }
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

pub fn query_cyberlink_by_formatted_id(deps: Deps, formatted_id: String) -> StdResult<CyberlinkState> {
    // First try to load directly from NAMED_CYBERLINKS
    let global_id = NAMED_CYBERLINKS.load(deps.storage, &formatted_id)?;
    if DELETED_IDS.has(deps.storage, global_id) {
        return Err(StdError::not_found("deleted cyberlink"));
    }

    let cyberlink_state = cyberlinks().load(deps.storage, global_id)?;

    Ok(cyberlink_state)
}

pub fn query_cyberlinks_set_by_ids(deps: Deps, ids: Vec<String>) -> StdResult<Vec<(String, CyberlinkState)>> {
    let mut links: Vec<(String, CyberlinkState)> = vec![];

    for formatted_id in ids {
        // Load the global ID corresponding to the formatted ID
        match NAMED_CYBERLINKS.load(deps.storage, &formatted_id) {
            Ok(global_id) => {
                // Check if the cyberlink is deleted
                if DELETED_IDS.has(deps.storage, global_id) {
                    continue; // Skip deleted cyberlinks
                }
                // Load the cyberlink state
                match cyberlinks().load(deps.storage, global_id) {
                    Ok(cyberlink_state) => {
                        links.push((formatted_id.clone(), cyberlink_state));
                    },
                    Err(_) => continue, // Skip if loading fails (should be rare if NAMED_CYBERLINKS exists)
                }
            },
            Err(_) => continue, // Skip if the formatted ID doesn't exist
        }
    }

    Ok(links)
}

pub fn query_cyberlinks_by_owner_and_type(
    deps: Deps,
    owner: String,
    type_: String,
    start_after: Option<u64>,
    limit: Option<u32>
) -> StdResult<Vec<(u64, CyberlinkState)>> {
    let owner_addr = deps.api.addr_validate(&owner)?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    cyberlinks()
        .idx
        .owner_type
        // Use prefix for the composite key (owner_addr, type_)
        .prefix((owner_addr, type_))
        .range(
            deps.storage,
            start, // The start_after (u64) refers to the primary key (GID)
            None,
            Order::Ascending,
        )
        .take(limit)
        .collect()
}

#[cw_serde]
pub struct StateResponse {
    pub cyberlinks: Vec<(u64, CyberlinkState)>,
    pub named_cyberlinks: Vec<(String, u64)>
}

// Tier 4 Query: Get Counts
pub fn query_get_counts(
    deps: Deps,
    owner: Option<String>,
    type_: Option<String>,
) -> StdResult<CountsResponse> {
    let mut response = CountsResponse {
        owner_count: None,
        type_count: None,
        owner_type_count: None,
    };

    // Load owner count if owner is specified
    let owner_addr_opt = owner.as_ref().map(|o| deps.api.addr_validate(o)).transpose()?;
    if let Some(ref owner_addr) = owner_addr_opt {
        response.owner_count = OWNER_LINK_COUNT.may_load(deps.storage, owner_addr)?.map(Uint64::new);
    }

    // Load type count if type is specified
    if let Some(ref type_str) = type_ {
        response.type_count = TYPE_LINK_COUNT.may_load(deps.storage, type_str)?.map(Uint64::new);
    }

    // Load owner-type count if both owner and type are specified
    if let (Some(ref owner_addr), Some(ref type_str)) = (owner_addr_opt, type_.as_ref()) {
        response.owner_type_count = OWNER_TYPE_LINK_COUNT
            .may_load(deps.storage, (owner_addr, type_str))?
            .map(Uint64::new);
    }

    Ok(response)
}