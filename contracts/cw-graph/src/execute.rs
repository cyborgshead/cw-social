use crate::contract::map_validate;
use crate::error::ContractError;
use crate::msg::Cyberlink;
use crate::state::{cyberlinks, CyberlinkState, CONFIG, DELETED_IDS, ID, NAMED_CYBERLINKS, TYPE_IDS, OWNER_LINK_COUNT, TYPE_LINK_COUNT, OWNER_TYPE_LINK_COUNT};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, Uint64, Storage, Addr, StdResult};

fn validate_cyberlink(
    deps: Deps,
    id: Option<String>,
    cyberlink: Cyberlink
) -> Result<(), ContractError> {
    // Validation
    if cyberlink.from != cyberlink.to && (cyberlink.from.is_none() || cyberlink.to.is_none()) {
        return Err(ContractError::InvalidCyberlink {
            id: Uint64::zero(),
            from: cyberlink.from.unwrap_or_else(|| "_".to_string()),
            to: cyberlink.to.unwrap_or_else(|| "_".to_string()),
            type_: cyberlink.type_.clone(),
        });
    }

    let (mut dfrom, mut dto): (Option<CyberlinkState>, Option<CyberlinkState>) = (None, None);

    let dtype_id = NAMED_CYBERLINKS.may_load(deps.storage, cyberlink.type_.as_str())?;
    if dtype_id.is_none() {
        return Err(ContractError::TypeNotExists { type_: cyberlink.type_.clone() });
    }
    let dtype = cyberlinks().load(deps.storage, dtype_id.unwrap()).unwrap();

    if cyberlink.from.is_some() {
        let dfrom_id = NAMED_CYBERLINKS.may_load(deps.storage, cyberlink.clone().from.unwrap().as_str())?;
        if dfrom_id.is_none() {
            return Err(ContractError::FromNotExists { from: cyberlink.from.unwrap_or_else(|| "_".to_string()) });
        }
        dfrom = cyberlinks().may_load(deps.storage, dfrom_id.unwrap()).unwrap();
    }
    if cyberlink.to.is_some() {
        let dto_id = NAMED_CYBERLINKS.may_load(deps.storage, cyberlink.clone().to.unwrap().as_str())?;
        if dto_id.is_none() {
            return Err(ContractError::ToNotExists { to: cyberlink.to.unwrap_or_else(|| "_".to_string()) });
        }
        dto = cyberlinks().may_load(deps.storage, dto_id.unwrap()).unwrap();
    }

    // Additional validation for type conflicts
    if let (Some(_), Some(_)) = (&cyberlink.from, &cyberlink.to) {
        if dtype.clone().from.ne(&"Any") && dtype.clone().from.ne(&dfrom.clone().unwrap().type_) {
            return Err(ContractError::TypeConflict {
                id: id.unwrap_or_else(|| "_".to_string()),
                type_: cyberlink.clone().type_,
                from: cyberlink.clone().from.unwrap_or_else(|| "_".to_string()),
                to: cyberlink.clone().to.unwrap_or_else(|| "_".to_string()),
                expected_type: cyberlink.clone().type_,
                expected_from: dtype.clone().from,
                expected_to: dtype.clone().to,
                received_type: cyberlink.clone().type_,
                received_from: dfrom.clone().unwrap().type_,
                received_to: dto.clone().unwrap().type_,
            });
        }

        if dtype.to.ne(&"Any") && dtype.to.ne(&dto.clone().unwrap().type_) {
            return Err(ContractError::TypeConflict {
                id: id.unwrap_or_else(|| "_".to_string()),
                type_: cyberlink.clone().type_,
                from: cyberlink.clone().from.unwrap_or_else(|| "_".to_string()),
                to: cyberlink.clone().to.unwrap_or_else(|| "_".to_string()),
                expected_type: cyberlink.clone().type_,
                expected_from: dtype.from,
                expected_to: dtype.to,
                received_type: cyberlink.clone().type_,
                received_from: dfrom.clone().unwrap().type_,
                received_to: dto.clone().unwrap().type_,
            });
        }
    }

    Ok(())
}

fn create_cyberlink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: Option<String>,
    cyberlink: Cyberlink
) -> Result<(u64, String), ContractError> {
    // Get next global ID for internal indexing
    let id = ID.load(deps.storage)? + 1;
    ID.save(deps.storage, &id)?;

    let formatted_id: String;
    if name.is_none() {
        // Get and increment the type-specific ID
        let type_id = TYPE_IDS.may_load(deps.storage, cyberlink.type_.as_str())?.unwrap_or(0) + 1;
        TYPE_IDS.save(deps.storage, cyberlink.type_.as_str(), &type_id)?;

        // Generate the formatted ID string (e.g., "post:42")
        formatted_id = format!("{}:{}", cyberlink.type_, type_id);
    } else {
        formatted_id = name.unwrap();
    }


    // Save new Cyberlink
    let cyberlink_state = CyberlinkState {
        type_: cyberlink.type_.clone(),
        from: cyberlink.from.unwrap_or_else(|| "Any".to_string()),
        to: cyberlink.to.unwrap_or_else(|| "Any".to_string()),
        value: cyberlink.value.unwrap_or_default(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
        formatted_id: Some(formatted_id.clone()),
    };

    // Also save the cyberlink with its string ID for direct access
    NAMED_CYBERLINKS.save(deps.storage, formatted_id.as_str(), &id)?;

    // Save the cyberlink using IndexedMap with numeric ID for efficient indexing
    cyberlinks().save(deps.storage, id, &cyberlink_state)?;

    // ---- Increment Counters ----
    increment_counters(deps.storage, &cyberlink_state.owner, &cyberlink_state.type_)?;
    // -------------------------

    Ok((id, formatted_id))
}

pub fn execute_create_named_cyberlink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    cyberlink: Cyberlink,
) -> Result<Response, ContractError> {
    // Check if the user is an admin
    let config = CONFIG.load(deps.storage)?;
    if !config.can_modify(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate name doesn't contain colons
    if name.contains(':') {
        return Err(ContractError::InvalidNameFormat { name });
    }

    // Validate the cyberlink
    validate_cyberlink(deps.as_ref(), None, cyberlink.clone())?;

    // Create the cyberlink
    let (numeric_id, formatted_id) = create_cyberlink(deps, env, info, Some(name), cyberlink.clone())?;

    Ok(Response::new()
        .add_attribute("action", "create_cyberlink")
        .add_attribute("numeric_id", numeric_id.to_string())
        .add_attribute("formatted_id", formatted_id)
        .add_attribute("type", cyberlink.type_)
    )
}

pub fn execute_create_cyberlink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cyberlink: Cyberlink
) -> Result<Response, ContractError> {
    // Check if the user is an executor
    let config = CONFIG.load(deps.storage)?;
    if !config.can_execute(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the cyberlink
    validate_cyberlink(deps.as_ref(), None, cyberlink.clone())?;

    // Create the cyberlink
    let (numeric_id, formatted_id) = create_cyberlink(deps, env, info, None, cyberlink.clone())?;

    Ok(Response::new()
        .add_attribute("action", "create_cyberlink")
        .add_attribute("numeric_id", numeric_id.to_string())
        .add_attribute("formatted_id", formatted_id)
        .add_attribute("type", cyberlink.type_)
    )
}

pub fn execute_create_cyberlinks(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cyberlinks: Vec<Cyberlink>
) -> Result<Response, ContractError> {
    // Check if the user is an executor
    let config = CONFIG.load(deps.storage)?;
    if !config.can_execute(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    let mut numeric_ids = Vec::with_capacity(cyberlinks.len());
    let mut formatted_ids = Vec::with_capacity(cyberlinks.len());
    
    for cyberlink in cyberlinks {
        // Validate the cyberlink
        validate_cyberlink(deps.as_ref(), None, cyberlink.clone())?;

        // Create the cyberlink (this now increments counters internally)
        let (numeric_id, formatted_id) = create_cyberlink(deps.branch(), env.clone(), info.clone(), None, cyberlink)?;
        numeric_ids.push(numeric_id);
        formatted_ids.push(formatted_id);
    }

    Ok(Response::new()
        .add_attribute("action", "create_cyberlinks")
        .add_attribute("count", numeric_ids.len().to_string())
        .add_attribute("numeric_ids", numeric_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","))
        .add_attribute("formatted_ids", formatted_ids.join(","))
    )
}

pub fn execute_update_cyberlink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: String,
    new_value: Option<String>, // Renamed parameter
) -> Result<Response, ContractError> {
    let global_id = NAMED_CYBERLINKS.may_load(deps.storage, id.as_str())?.ok_or_else(|| ContractError::NotFound { id: id.clone() })?;

    let deleted_id = DELETED_IDS.may_load(deps.storage, global_id)?;
    if deleted_id.is_some() {
        return Err(ContractError::DeletedCyberlink { id });
    }

    // Check if the cyberlink exists and load old state
    let old_cyberlink_state = cyberlinks().load(deps.storage, global_id)?;

    let config = CONFIG.load(deps.storage)?;

    // Check if the user is the owner or an admin
    if old_cyberlink_state.owner != info.sender && !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Update the state
    let mut updated_state = old_cyberlink_state; // Use old state directly
    updated_state.value = new_value.unwrap_or_default(); // Update value
    updated_state.updated_at = Some(env.block.time); // Set updated time

    // Save updated state
    cyberlinks().save(deps.storage, global_id, &updated_state)?;

    Ok(Response::new()
        .add_attribute("action", "update_cyberlink")
        .add_attribute("numeric_id", global_id.to_string())
        .add_attribute("formatted_id", id)
    )
}

pub fn execute_delete_cyberlink(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: String // Formatted ID (e.g., "Type:1")
) -> Result<Response, ContractError> {
    // Load the global ID corresponding to the formatted ID
    let global_id = NAMED_CYBERLINKS.may_load(deps.storage, id.as_str())?.ok_or_else(|| ContractError::NotFound { id: id.clone() })?;

    // Check if already marked as deleted
    if DELETED_IDS.has(deps.storage, global_id) {
        return Err(ContractError::DeletedCyberlink { id });
    }

    // Load the cyberlink state to check ownership and get details for counter decrement
    let cyberlink_state = cyberlinks().load(deps.storage, global_id)?;

    let config = CONFIG.load(deps.storage)?;

    // Check if the user is the owner or an admin
    if cyberlink_state.owner != info.sender && !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // ---- Decrement Counters ----
    decrement_counters(deps.storage, &cyberlink_state.owner, &cyberlink_state.type_)?;
    // -------------------------

    // Mark the cyberlink as deleted using the DELETED_IDS map
    DELETED_IDS.save(deps.storage, global_id, &true)?;

    // Optional: Completely remove the cyberlink state and its named entry to save space
    cyberlinks().remove(deps.storage, global_id)?;
    // NAMED_CYBERLINKS.remove(deps.storage, id.as_str());
    // Consider the implications: Queries by GID will fail entirely instead of returning a "deleted" error.
    // Queries relying on the existence of the NAMED_CYBERLINKS entry will also fail.

    Ok(Response::new()
        .add_attribute("action", "delete_cyberlink")
        .add_attribute("numeric_id", global_id.to_string())
        .add_attribute("formatted_id", id)
    )
}

pub fn execute_update_admins(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admins: Vec<String>,
) -> Result<Response, ContractError> {
    // Load config
    let mut config = CONFIG.load(deps.storage)?;

    // Check if the user is an admin
    if !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Update admins
    config.admins = map_validate(deps.api, &new_admins)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_admins")
        .add_attribute("count", new_admins.len().to_string())
    )
}

pub fn execute_update_executors(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_executors: Vec<String>,
) -> Result<Response, ContractError> {
    // Load config
    let mut config = CONFIG.load(deps.storage)?;

    // Check if the user is an admin
    if !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Update executors
    config.executors = map_validate(deps.api, &new_executors)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "update_executors")
        .add_attribute("count", new_executors.len().to_string())
    )
}

// --- Counter Helper Functions ---

fn increment_counters(
    storage: &mut dyn Storage,
    owner: &Addr,
    type_: &str,
) -> StdResult<()> {
    // Increment owner count
    let owner_count = OWNER_LINK_COUNT.may_load(storage, owner)?.unwrap_or(0) + 1;
    OWNER_LINK_COUNT.save(storage, owner, &owner_count)?;

    // Increment type count
    let type_count = TYPE_LINK_COUNT.may_load(storage, type_)?.unwrap_or(0) + 1;
    TYPE_LINK_COUNT.save(storage, type_, &type_count)?;

    // Increment owner-type count
    let owner_type_count = OWNER_TYPE_LINK_COUNT.may_load(storage, (owner, type_))?.unwrap_or(0) + 1;
    OWNER_TYPE_LINK_COUNT.save(storage, (owner, type_), &owner_type_count)?;

    Ok(())
}

fn decrement_counters(
    storage: &mut dyn Storage,
    owner: &Addr,
    type_: &str,
) -> StdResult<()> {
    // Decrement owner count, removing if zero
    let owner_count = OWNER_LINK_COUNT.load(storage, owner)?;
    if owner_count <= 1 {
        OWNER_LINK_COUNT.remove(storage, owner);
    } else {
        OWNER_LINK_COUNT.save(storage, owner, &(owner_count - 1))?;
    }

    // Decrement type count, removing if zero
    let type_count = TYPE_LINK_COUNT.load(storage, type_)?;
    if type_count <= 1 {
        TYPE_LINK_COUNT.remove(storage, type_);
    } else {
        TYPE_LINK_COUNT.save(storage, type_, &(type_count - 1))?;
    }

    // Decrement owner-type count, removing if zero
    let owner_type_count = OWNER_TYPE_LINK_COUNT.load(storage, (owner, type_))?;
    if owner_type_count <= 1 {
        OWNER_TYPE_LINK_COUNT.remove(storage, (owner, type_));
    } else {
        OWNER_TYPE_LINK_COUNT.save(storage, (owner, type_), &(owner_type_count - 1))?;
    }

    Ok(())
}

