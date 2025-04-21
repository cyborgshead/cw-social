use crate::contract::map_validate;
use crate::error::ContractError;
use crate::msg::Cyberlink;
use crate::state::{cyberlinks, CyberlinkState, CONFIG, DELETED_GIDS, GID, NAMED_CYBERLINKS, TYPE_GIDS, OWNER_LINK_COUNT, TYPE_LINK_COUNT, OWNER_TYPE_LINK_COUNT};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, Uint64, Storage, Addr, StdResult};

fn validate_cyberlink(
    deps: Deps,
    fid: Option<String>,
    cyberlink: Cyberlink
) -> Result<(), ContractError> {
    // Validation
    if cyberlink.from != cyberlink.to && (cyberlink.from.is_none() || cyberlink.to.is_none()) {
        return Err(ContractError::InvalidCyberlink {
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
    let id = GID.load(deps.storage)? + 1;
    GID.save(deps.storage, &id)?;

    let formatted_id: String;
    if name.is_none() {
        // Get and increment the type-specific ID
        let type_id = TYPE_GIDS.may_load(deps.storage, cyberlink.type_.as_str())?.unwrap_or(0) + 1;
        TYPE_GIDS.save(deps.storage, cyberlink.type_.as_str(), &type_id)?;

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
        fid: Some(formatted_id.clone()),
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
        .add_attribute("gid", numeric_id.to_string())
        .add_attribute("fid", formatted_id)
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
        .add_attribute("type", cyberlink.type_)
        .add_attribute("gid", numeric_id.to_string())
        .add_attribute("fid", formatted_id)
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

    let mut gids = Vec::with_capacity(cyberlinks.len());
    let mut fids = Vec::with_capacity(cyberlinks.len());
    
    for cyberlink in cyberlinks {
        // Validate the cyberlink
        validate_cyberlink(deps.as_ref(), None, cyberlink.clone())?;

        // Create the cyberlink (this now increments counters internally)
        let (gid, fid) = create_cyberlink(deps.branch(), env.clone(), info.clone(), None, cyberlink)?;
        gids.push(gid);
        fids.push(fid);
    }

    Ok(Response::new()
        .add_attribute("action", "create_cyberlinks")
        .add_attribute("count", gids.len().to_string())
        .add_attribute("gids", gids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(","))
        .add_attribute("fids", fids.join(","))
    )
}

pub fn execute_update_cyberlink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    fid: String,
    new_value: Option<String>, // Renamed parameter
) -> Result<Response, ContractError> {
    let gid = NAMED_CYBERLINKS.may_load(deps.storage, fid.as_str())?.ok_or_else(|| ContractError::NotFound { fid: fid.clone() })?;

    let deleted_id = DELETED_GIDS.may_load(deps.storage, gid)?;
    if deleted_id.is_some() {
        return Err(ContractError::DeletedCyberlink { fid });
    }

    // Check if the cyberlink exists and load old state
    let old_cyberlink_state = cyberlinks().load(deps.storage, gid)?;

    let config = CONFIG.load(deps.storage)?;

    // Check if the user is the owner or an admin
    if old_cyberlink_state.owner != info.sender && !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Update the state and save
    cyberlinks().update(deps.storage, gid, |old_opt| -> Result<CyberlinkState, ContractError> {
        let mut state = old_opt.ok_or_else(|| ContractError::NotFound { fid: fid.clone() })?;
        state.value = new_value.unwrap_or_default(); // Update value
        state.updated_at = Some(env.block.time); // Set updated time
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_cyberlink")
        .add_attribute("gid", gid.to_string())
        .add_attribute("fid", fid)
    )
}

pub fn execute_delete_cyberlink(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    fid: String // Formatted ID (e.g., "Type:1")
) -> Result<Response, ContractError> {
    // Load the global ID corresponding to the formatted ID
    let gid = NAMED_CYBERLINKS.may_load(deps.storage, fid.as_str())?.ok_or_else(|| ContractError::NotFound { fid: fid.clone() })?;

    // Check if already marked as deleted
    if DELETED_GIDS.has(deps.storage, gid) {
        return Err(ContractError::DeletedCyberlink { fid: fid });
    }

    // Load the cyberlink state to check ownership and get details for counter decrement
    let cyberlink_state = cyberlinks().load(deps.storage, gid)?;

    let config = CONFIG.load(deps.storage)?;

    // Check if the user is the owner or an admin
    if cyberlink_state.owner != info.sender && !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // ---- Decrement Counters ----
    decrement_counters(deps.storage, &cyberlink_state.owner, &cyberlink_state.type_)?;
    // -------------------------

    // Mark the cyberlink as deleted using the DELETED_IDS map
    DELETED_GIDS.save(deps.storage, gid, &true)?;

    // Optional: Completely remove the cyberlink state and its named entry to save space
    cyberlinks().remove(deps.storage, gid)?;
    // NAMED_CYBERLINKS.remove(deps.storage, id.as _str());
    // Consider the implications: Queries by GID will fail entirely instead of returning a "deleted" error.
    // Queries relying on the existence of the NAMED_CYBERLINKS entry will also fail.

    Ok(Response::new()
        .add_attribute("action", "delete_cyberlink")
        .add_attribute("gid", gid.to_string())
        .add_attribute("fid", fid)
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

fn validate_type_compatibility_for_cyberlink2(
    link_type_state: &CyberlinkState,
    node_type: &str, // Type of the node node being created
    existing_node_state: &CyberlinkState, // State of the existing node being linked
    link_from_new: bool, // True if the link is from the new node, false if to the new node
    existing_node_fid: &str, // FID of the existing node for error reporting
) -> Result<(), ContractError> {
    if link_from_new { // Link: New -> Existing
        // Check link_type's 'from' constraint against the new node's type
        if link_type_state.from != "Any" && link_type_state.from != node_type {
            return Err(ContractError::TypeConflict {
                type_: link_type_state.type_.clone(),
                from: "<new_node>".to_string(), // Placeholder as new FID isn't known yet
                to: existing_node_fid.to_string(),
                expected_type: link_type_state.type_.clone(),
                expected_from: link_type_state.from.clone(),
                expected_to: link_type_state.to.clone(),
                received_type: link_type_state.type_.clone(),
                received_from: node_type.to_string(),
                received_to: existing_node_state.type_.clone(),
            });
        }
        // Check link_type's 'to' constraint against the existing node's type
        if link_type_state.to != "Any" && link_type_state.to != existing_node_state.type_ {
            return Err(ContractError::TypeConflict {
                type_: link_type_state.type_.clone(),
                from: "<new_node>".to_string(),
                to: existing_node_fid.to_string(),
                expected_type: link_type_state.type_.clone(),
                expected_from: link_type_state.from.clone(),
                expected_to: link_type_state.to.clone(),
                received_type: link_type_state.type_.clone(),
                received_from: node_type.to_string(),
                received_to: existing_node_state.type_.clone(),
            });
        }
    } else { // Link: Existing -> New
        // Check link_type's 'from' constraint against the existing node's type
        if link_type_state.from != "Any" && link_type_state.from != existing_node_state.type_ {
             return Err(ContractError::TypeConflict {
                type_: link_type_state.type_.clone(),
                from: existing_node_fid.to_string(),
                to: "<new_node>".to_string(),
                expected_type: link_type_state.type_.clone(),
                expected_from: link_type_state.from.clone(),
                expected_to: link_type_state.to.clone(),
                received_type: link_type_state.type_.clone(),
                received_from: existing_node_state.type_.clone(),
                received_to: node_type.to_string(),
             });
        }
        // Check link_type's 'to' constraint against the new node's type
        if link_type_state.to != "Any" && link_type_state.to != node_type {
             return Err(ContractError::TypeConflict {
                type_: link_type_state.type_.clone(),
                from: existing_node_fid.to_string(),
                to: "<new_node>".to_string(),
                expected_type: link_type_state.type_.clone(),
                expected_from: link_type_state.from.clone(),
                expected_to: link_type_state.to.clone(),
                received_type: link_type_state.type_.clone(),
                received_from: existing_node_state.type_.clone(),
                received_to: node_type.to_string(),
             });
        }
    }
    Ok(())
}

pub fn execute_create_cyberlink2(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    node_type: String,
    node_value: Option<String>,
    link_type: String,
    link_value: Option<String>,
    link_from_existing_id: Option<String>,
    link_to_existing_id: Option<String>,
) -> Result<Response, ContractError> {
    // Input Validation and Link Specification
    let (existing_node_fid, link_from_new, _link_to_new) = // Renamed link_to_new as it's unused after this block
        match (link_from_existing_id.clone(), link_to_existing_id.clone()) {
            (Some(from_id), None) => (from_id, false, true), // Link FROM existing TO new
            (None, Some(to_id)) => (to_id, true, false),   // Link FROM new TO existing
            _ => return Err(ContractError::InvalidLinkSpecification {}),
        };

    // Type Existence
    let _node_type_gid = NAMED_CYBERLINKS.may_load(deps.storage, &node_type)? // Renamed to avoid shadowing
        .ok_or_else(|| ContractError::TypeNotExists { type_: node_type.clone() })?;
    // Load node type state only if needed for strict validation later, otherwise existence check is enough
    // let _node_type_state = cyberlinks().load(deps.storage, node_type_gid)?;
    
    let link_type_gid = NAMED_CYBERLINKS.may_load(deps.storage, &link_type)?
        .ok_or_else(|| ContractError::TypeNotExists { type_: link_type.clone() })?;
    let link_type_state = cyberlinks().load(deps.storage, link_type_gid)?;

    // Existing Node Validation
    let existing_node_gid = NAMED_CYBERLINKS.may_load(deps.storage, &existing_node_fid)?
        .ok_or_else(|| {
            if link_from_new { // Linking TO existing, so it's a 'ToNotExists' error
                ContractError::ToNotExists { to: existing_node_fid.clone() }
            } else { // Linking FROM existing, so it's a 'FromNotExists' error
                ContractError::FromNotExists { from: existing_node_fid.clone() }
            }
        })?;
    
    if DELETED_GIDS.has(deps.storage, existing_node_gid) {
        return Err(ContractError::NotFound { fid: existing_node_fid.clone() }); // Treat deleted as not found for linking
    }
    let existing_node_state = cyberlinks().load(deps.storage, existing_node_gid)?;

    // Type Compatibility Validation (Using loaded states)
    validate_type_compatibility_for_cyberlink2(
        &link_type_state,
        &node_type,
        &existing_node_state,
        link_from_new,
        &existing_node_fid,
    )?;

    // Create New Node
    let node_cyberlink = Cyberlink {
        type_: node_type, // Use validated node_type
        from: None, // Vertices are nodes, no from/to
        to: None,
        value: node_value,
    };
    // Use deps.branch() for the first creation to isolate potential state changes if create_cyberlink modified more state
    let (node_gid, node_fid) = 
        create_cyberlink(deps.branch(), env.clone(), info.clone(), None, node_cyberlink)?;

    // 4. Create Link
    let (link_from, link_to) = if link_from_new {
        (Some(node_fid.clone()), Some(existing_node_fid.clone()))
    } else {
        (Some(existing_node_fid.clone()), Some(node_fid.clone()))
    };

    let link_cyberlink = Cyberlink {
        type_: link_type, // Use validated link_type
        from: link_from,
        to: link_to,
        value: link_value,
    };
    // Don't need validate_cyberlink here as create_cyberlink does necessary checks (like type existence)
    // and we performed the complex logic checks (like type compatibility) already.
    let (link_gid, link_fid) = 
        create_cyberlink(deps, env, info, None, link_cyberlink)?;

    // 5. Response
    Ok(Response::new()
        .add_attribute("action", "create_cyberlink2")
        .add_attribute("node_gid", node_gid.to_string())
        .add_attribute("node_fid", node_fid)
        .add_attribute("link_gid", link_gid.to_string())
        .add_attribute("link_fid", link_fid)
    )
}

