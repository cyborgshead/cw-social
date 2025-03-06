use cosmwasm_std::{attr, Deps, DepsMut, Env, MessageInfo, Order, Response, StdResult, SubMsg, Uint64};
use cosmwasm_std::Order::Ascending;
use cw_storage_plus::Bound;
use crate::error::ContractError;
use crate::state::{CONFIG, DeeplinkState, ID, DELETED_IDS, NAMED_DEEPLINKS, deeplinks};
use crate::contract::map_validate;
use crate::msg::Deeplink;

fn validate_deeplink(
    deps: Deps,
    id: Option<String>,
    deeplink: Deeplink
) -> Result<(), ContractError> {
    // Validation
    if deeplink.from != deeplink.to && (deeplink.from.is_none() || deeplink.to.is_none()) {
        return Err(ContractError::InvalidDeeplink {
            id: Uint64::zero(),
            from: deeplink.from.unwrap_or_else(|| "_".to_string()),
            to: deeplink.to.unwrap_or_else(|| "_".to_string()),
            type_: deeplink.type_.clone(),
        });
    }

    let (mut dtype_, mut dfrom, mut dto): (Option<DeeplinkState>, Option<DeeplinkState>, Option<DeeplinkState>) = (None, None, None);

    dtype_ = NAMED_DEEPLINKS.may_load(deps.storage, deeplink.type_.as_str())?;
    if dtype_.is_none() {
        return Err(ContractError::TypeNotExists { type_: deeplink.type_.clone() });
    }
    if deeplink.from.is_some() {
        dfrom = NAMED_DEEPLINKS.may_load(deps.storage, deeplink.clone().from.unwrap().as_str())?;
        if dfrom.is_none() {
            return Err(ContractError::FromNotExists { from: deeplink.from.unwrap_or_else(|| "_".to_string()) });
        }
    }
    if deeplink.to.is_some() {
        dto = NAMED_DEEPLINKS.may_load(deps.storage, deeplink.clone().to.unwrap().as_str())?;
        if dto.is_none() {
            return Err(ContractError::ToNotExists { to: deeplink.to.unwrap_or_else(|| "_".to_string()) });
        }
    }

    // Additional validation for type conflicts
    if let (Some(ref from), Some(ref to)) = (&deeplink.from, &deeplink.to) {
        if dtype_.clone().unwrap().from.ne(&"Any") && dtype_.clone().unwrap().from.ne(&dfrom.clone().unwrap().type_) {
            return Err(ContractError::TypeConflict {
                id: id.unwrap_or_else(|| "_".to_string()),
                type_: deeplink.clone().type_,
                from: deeplink.clone().from.unwrap_or_else(|| "_".to_string()),
                to: deeplink.clone().to.unwrap_or_else(|| "_".to_string()),
                expected_type: deeplink.clone().type_,
                expected_from: dtype_.clone().unwrap().from,
                expected_to: dtype_.clone().unwrap().to,
                received_type: deeplink.clone().type_,
                received_from: dfrom.clone().unwrap().type_,
                received_to: dto.clone().unwrap().type_,
            });
        }

        if dtype_.clone().unwrap().to.ne(&"Any") && dtype_.clone().unwrap().to.ne(&dto.clone().unwrap().type_) {
            return Err(ContractError::TypeConflict {
                id: id.unwrap_or_else(|| "_".to_string()),
                type_: deeplink.clone().type_,
                from: deeplink.clone().from.unwrap_or_else(|| "_".to_string()),
                to: deeplink.clone().to.unwrap_or_else(|| "_".to_string()),
                expected_type: deeplink.clone().type_,
                expected_from: dtype_.clone().unwrap().from,
                expected_to: dtype_.clone().unwrap().to,
                received_type: deeplink.clone().type_,
                received_from: dfrom.clone().unwrap().type_,
                received_to: dto.clone().unwrap().type_,
            });
        }
    }

    Ok(())
}

fn create_deeplink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    deeplink: Deeplink
) -> Result<u64, ContractError> {
    // Get next ID
    let id = ID.load(deps.storage)? + 1;
    ID.save(deps.storage, &id)?;

    // Save new Deeplink
    let deeplink_state = DeeplinkState {
        type_: deeplink.type_.clone(),
        from: deeplink.from.unwrap_or_else(|| "Any".to_string()),
        to: deeplink.to.unwrap_or_else(|| "Any".to_string()),
        value: deeplink.value.unwrap_or_default(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
    };

    // Save the deeplink using IndexedMap
    // This will automatically update all indexes
    deeplinks().save(deps.storage, id, &deeplink_state)?;

    Ok(id)
}

pub fn execute_create_named_deeplink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    deeplink: Deeplink,
) -> Result<Response, ContractError> {
    // Check if the user is an admin
    let config = CONFIG.load(deps.storage)?;
    if !config.can_modify(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the deeplink
    validate_deeplink(deps.as_ref(), Some(name.clone()), deeplink.clone())?;

    // Save new Deeplink
    // let type_ = deeplink.clone().type_;
    let deeplink_state = DeeplinkState {
        type_: deeplink.clone().type_,
        from: deeplink.clone().from.unwrap_or_else(|| "Any".to_string()),
        to: deeplink.clone().to.unwrap_or_else(|| "Any".to_string()),
        value: deeplink.value.unwrap_or_default(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
    };

    // Save the named deeplink
    NAMED_DEEPLINKS.save(deps.storage, name.as_str(), &deeplink_state)?;

    Ok(Response::new()
        .add_attribute("action", "create_named_deeplink")
        .add_attribute("name", name)
        .add_attribute("type", deeplink.type_)
    )
}

pub fn execute_create_deeplink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    deeplink: Deeplink
) -> Result<Response, ContractError> {
    // Check if the user is an executor
    let config = CONFIG.load(deps.storage)?;
    if !config.can_execute(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the deeplink
    validate_deeplink(deps.as_ref(), None, deeplink.clone())?;

    // Create the deeplink
    let id = create_deeplink(deps, env, info, deeplink.clone())?;

    Ok(Response::new()
        .add_attribute("action", "create_deeplink")
        .add_attribute("id", id.to_string())
        .add_attribute("type", deeplink.type_)
    )
}

pub fn execute_create_deeplinks(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    deeplinks: Vec<Deeplink>
) -> Result<Response, ContractError> {
    // Check if the user is an executor
    let config = CONFIG.load(deps.storage)?;
    if !config.can_execute(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    let mut ids = Vec::with_capacity(deeplinks.len());
    for deeplink in deeplinks {
        // Validate the deeplink
        validate_deeplink(deps.as_ref(), None, deeplink.clone())?;

        // Create the deeplink
        let id = create_deeplink(deps.branch(), env.clone(), info.clone(), deeplink)?;
        ids.push(id);
    }

    Ok(Response::new()
        .add_attribute("action", "create_deeplinks")
        .add_attribute("count", ids.len().to_string())
    )
}

pub fn execute_update_deeplink(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: u64,
    deeplink: Deeplink,
) -> Result<Response, ContractError> {
    // Check if the user is an executor
    let config = CONFIG.load(deps.storage)?;
    if !config.can_execute(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Check if the deeplink exists
    let mut deeplink_state = deeplinks().load(deps.storage, id)?;

    // Check if the user is the owner or an admin
    if deeplink_state.owner != info.sender && !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the deeplink
    validate_deeplink(deps.as_ref(), Some(id.to_string()), deeplink.clone())?;

    // Update the deeplink
    deeplink_state.type_ = deeplink.type_.clone();
    deeplink_state.from = deeplink.from.unwrap_or_else(|| "Any".to_string());
    deeplink_state.to = deeplink.to.unwrap_or_else(|| "Any".to_string());
    deeplink_state.value = deeplink.value.unwrap_or_default();
    deeplink_state.updated_at = Some(env.block.time);

    // Save the updated deeplink
    // This will automatically update all indexes
    deeplinks().save(deps.storage, id, &deeplink_state)?;

    Ok(Response::new()
        .add_attribute("action", "update_deeplink")
        .add_attribute("id", id.to_string())
    )
}

pub fn execute_delete_deeplink(
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

    // Check if the deeplink exists
    let deeplink_state = deeplinks().load(deps.storage, id.u64())?;

    // Check if the user is the owner or an admin
    if deeplink_state.owner != info.sender && !config.is_admin(info.sender.as_str()) {
        return Err(ContractError::Unauthorized {});
    }

    // Mark the deeplink as deleted
    DELETED_IDS.save(deps.storage, id.u64(), &true)?;

    // Remove the deeplink from the IndexedMap
    // This will automatically remove all indexes
    deeplinks().remove(deps.storage, id.u64())?;

    Ok(Response::new()
        .add_attribute("action", "delete_deeplink")
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

