use cosmwasm_std::{attr, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, SubMsg, Uint64};
use cosmwasm_std::Order::Ascending;
use cw_storage_plus::Bound;
use crate::error::ContractError;
use crate::state::{CONFIG, CyberlinkState, ID, DELETED_IDS, NAMED_CYBERLINKS, cyberlinks};
use crate::contract::map_validate;
use crate::msg::Cyberlink;

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

    let (mut dtype_, mut dfrom, mut dto): (Option<CyberlinkState>, Option<CyberlinkState>, Option<CyberlinkState>) = (None, None, None);

    dtype_ = NAMED_CYBERLINKS.may_load(deps.storage, cyberlink.type_.as_str())?;
    if dtype_.is_none() {
        return Err(ContractError::TypeNotExists { type_: cyberlink.type_.clone() });
    }
    if cyberlink.from.is_some() {
        dfrom = NAMED_CYBERLINKS.may_load(deps.storage, cyberlink.clone().from.unwrap().as_str())?;
        if dfrom.is_none() {
            return Err(ContractError::FromNotExists { from: cyberlink.from.unwrap_or_else(|| "_".to_string()) });
        }
    }
    if cyberlink.to.is_some() {
        dto = NAMED_CYBERLINKS.may_load(deps.storage, cyberlink.clone().to.unwrap().as_str())?;
        if dto.is_none() {
            return Err(ContractError::ToNotExists { to: cyberlink.to.unwrap_or_else(|| "_".to_string()) });
        }
    }

    // Additional validation for type conflicts
    if let (Some(ref from), Some(ref to)) = (&cyberlink.from, &cyberlink.to) {
        if dtype_.clone().unwrap().from.ne(&"Any") && dtype_.clone().unwrap().from.ne(&dfrom.clone().unwrap().type_) {
            return Err(ContractError::TypeConflict {
                id: id.unwrap_or_else(|| "_".to_string()),
                type_: cyberlink.clone().type_,
                from: cyberlink.clone().from.unwrap_or_else(|| "_".to_string()),
                to: cyberlink.clone().to.unwrap_or_else(|| "_".to_string()),
                expected_type: cyberlink.clone().type_,
                expected_from: dtype_.clone().unwrap().from,
                expected_to: dtype_.clone().unwrap().to,
                received_type: cyberlink.clone().type_,
                received_from: dfrom.clone().unwrap().type_,
                received_to: dto.clone().unwrap().type_,
            });
        }

        if dtype_.clone().unwrap().to.ne(&"Any") && dtype_.clone().unwrap().to.ne(&dto.clone().unwrap().type_) {
            return Err(ContractError::TypeConflict {
                id: id.unwrap_or_else(|| "_".to_string()),
                type_: cyberlink.clone().type_,
                from: cyberlink.clone().from.unwrap_or_else(|| "_".to_string()),
                to: cyberlink.clone().to.unwrap_or_else(|| "_".to_string()),
                expected_type: cyberlink.clone().type_,
                expected_from: dtype_.clone().unwrap().from,
                expected_to: dtype_.clone().unwrap().to,
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
    cyberlink: Cyberlink
) -> Result<u64, ContractError> {
    // Get next ID
    let id = ID.load(deps.storage)? + 1;
    ID.save(deps.storage, &id)?;

    // Save new Cyberlink
    let cyberlink_state = CyberlinkState {
        type_: cyberlink.type_.clone(),
        from: cyberlink.from.unwrap_or_else(|| "Any".to_string()),
        to: cyberlink.to.unwrap_or_else(|| "Any".to_string()),
        value: cyberlink.value.unwrap_or_default(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
    };

    // Save the cyberlink using IndexedMap
    // This will automatically update all indexes
    cyberlinks().save(deps.storage, id, &cyberlink_state)?;

    Ok(id)
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

    // Validate the cyberlink
    validate_cyberlink(deps.as_ref(), Some(name.clone()), cyberlink.clone())?;

    // Save new Cyberlink
    // let type_ = cyberlink.clone().type_;
    let cyberlink_state = CyberlinkState {
        type_: cyberlink.clone().type_,
        from: cyberlink.clone().from.unwrap_or_else(|| "Any".to_string()),
        to: cyberlink.clone().to.unwrap_or_else(|| "Any".to_string()),
        value: cyberlink.value.unwrap_or_default(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
    };

    // Save the named cyberlink
    NAMED_CYBERLINKS.save(deps.storage, name.as_str(), &cyberlink_state)?;

    Ok(Response::new()
        .add_attribute("action", "create_named_cyberlink")
        .add_attribute("name", name)
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
    let id = create_cyberlink(deps, env, info, cyberlink.clone())?;

    Ok(Response::new()
        .add_attribute("action", "create_cyberlink")
        .add_attribute("id", id.to_string())
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

    let mut ids = Vec::with_capacity(cyberlinks.len());
    for cyberlink in cyberlinks {
        // Validate the cyberlink
        validate_cyberlink(deps.as_ref(), None, cyberlink.clone())?;

        // Create the cyberlink
        let id = create_cyberlink(deps.branch(), env.clone(), info.clone(), cyberlink)?;
        ids.push(id);
    }

    Ok(Response::new()
        .add_attribute("action", "create_cyberlinks")
        .add_attribute("count", ids.len().to_string())
    )
}

pub fn execute_update_cyberlink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    cyberlink: Cyberlink,
) -> Result<Response, ContractError> {
    // Check if the user is an executor
    let config = CONFIG.load(deps.storage)?;
    if !config.can_execute(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Check if the cyberlink exists
    let mut cyberlink_state = cyberlinks().load(deps.storage, id)?;

    // Check if the user is the owner or an admin
    if cyberlink_state.owner != info.sender && !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the cyberlink
    validate_cyberlink(deps.as_ref(), Some(id.to_string()), cyberlink.clone())?;

    // Update the cyberlink
    cyberlink_state.type_ = cyberlink.type_.clone();
    cyberlink_state.from = cyberlink.from.unwrap_or_else(|| "Any".to_string());
    cyberlink_state.to = cyberlink.to.unwrap_or_else(|| "Any".to_string());
    cyberlink_state.value = cyberlink.value.unwrap_or_default();
    cyberlink_state.updated_at = Some(env.block.time);

    // Save the updated cyberlink
    // This will automatically update all indexes
    cyberlinks().save(deps.storage, id, &cyberlink_state)?;

    Ok(Response::new()
        .add_attribute("action", "update_cyberlink")
        .add_attribute("id", id.to_string())
    )
}

pub fn execute_delete_cyberlink(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: Uint64
) -> Result<Response, ContractError> {
    // Check if the user is an admin
    let config = CONFIG.load(deps.storage)?;
    if !config.can_modify(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Check if the cyberlink exists
    let cyberlink_state = cyberlinks().load(deps.storage, id.u64())?;

    // Check if the user is the owner or an admin
    if cyberlink_state.owner != info.sender && !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Mark the cyberlink as deleted
    DELETED_IDS.save(deps.storage, id.u64(), &true)?;

    // Remove the cyberlink from the IndexedMap
    // This will automatically remove all indexes
    cyberlinks().remove(deps.storage, id.u64())?;

    Ok(Response::new()
        .add_attribute("action", "delete_cyberlink")
        .add_attribute("id", id.to_string())
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

