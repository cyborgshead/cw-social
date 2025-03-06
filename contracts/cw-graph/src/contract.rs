#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, StdResult, MessageInfo, Reply, Api, Addr, Empty, Response};
use cw2::{get_contract_version, set_contract_version};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, DeeplinkState, ID, NAMED_DEEPLINKS, deeplinks};
use crate::execute::{execute_create_deeplink, execute_delete_deeplink, execute_update_deeplink, execute_update_admins, execute_update_executors, execute_create_deeplinks, execute_create_named_deeplink};
use crate::query::{query_config, query_deeplinks, query_deeplinks_by_ids, query_id, query_last_id, query_named_deeplinks, query_state, query_deeplinks_by_owner, query_deeplinks_by_owner_time, query_deeplinks_by_owner_time_any};
use semver::Version;

const CONTRACT_NAME: &str = "crates.io:cw-graph";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        admins: map_validate(deps.api, &msg.admins)?,
        executors: map_validate(deps.api, &msg.executers)?,
    };
    CONFIG.save(deps.storage, &config)?;

    ID.save(deps.storage, &0)?;

    let id = ID.load(deps.storage)? + 1;
    ID.save(deps.storage, &id)?;
    deeplinks().save(deps.storage, id, &DeeplinkState {
        type_: "Type".to_string(),
        from: "Any".to_string(),
        to: "Any".to_string(),
        value: "".to_string(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
    })?;
    NAMED_DEEPLINKS.save(deps.storage,
                         "Type", &DeeplinkState {
            type_: "Type".to_string(),
            from: "Any".to_string(),
            to: "Any".to_string(),
            value: "".to_string(),
            owner: info.sender.clone(),
            created_at: env.block.time,
            updated_at: None,
        })?;

    let id = ID.load(deps.storage)? + 1;
    ID.save(deps.storage, &id)?;
    deeplinks().save(deps.storage, id, &DeeplinkState {
        type_: "Any".to_string(),
        from: "Null".to_string(),
        to: "Null".to_string(),
        value: "".to_string(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
    })?;
    NAMED_DEEPLINKS.save(deps.storage,
                         "Any", &DeeplinkState {
            type_: "Type".to_string(),
            from: "Null".to_string(),
            to: "Null".to_string(),
            value: "".to_string(),
            owner: info.sender.clone(),
            created_at: env.block.time,
            updated_at: None,
        })?;

    Ok(Response::default())
}

pub fn map_validate(api: &dyn Api, admins: &[String]) -> StdResult<Vec<Addr>> {
    admins.iter().map(|addr| api.addr_validate(addr)).collect()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateNamedDeeplink { name, deeplink } => execute_create_named_deeplink(deps, env, info, name, deeplink),
        ExecuteMsg::CreateDeeplink { deeplink } => execute_create_deeplink(deps, env, info, deeplink),
        ExecuteMsg::CreateDeeplinks { deeplinks } => execute_create_deeplinks(deps, env, info, deeplinks),
        ExecuteMsg::UpdateDeeplink { id, deeplink } => execute_update_deeplink(deps, env, info, id, deeplink),
        ExecuteMsg::DeleteDeeplink { id } => execute_delete_deeplink(deps, env, info, id),
        ExecuteMsg::UpdateAdmins { new_admins } => execute_update_admins(deps, env, info, new_admins),
        ExecuteMsg::UpdateExecutors { new_executors } => execute_update_executors(deps, env, info, new_executors)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastId {} => to_binary(&query_last_id(deps)?),
        QueryMsg::DebugState {} => to_binary(&query_state(deps)?),
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Deeplink { id } => to_binary(&query_id(deps, id)?),
        QueryMsg::Deeplinks { start_after, limit} => to_binary(&query_deeplinks(deps, start_after, limit)?),
        QueryMsg::DeeplinksByIds { ids } => to_binary(&query_deeplinks_by_ids(deps, ids)?),
        QueryMsg::NamedDeeplinks { start_after, limit } => to_binary(&query_named_deeplinks(deps, start_after, limit)?),
        QueryMsg::DeeplinksByOwner { owner, start_after, limit } => to_binary(&query_deeplinks_by_owner(deps, owner, start_after, limit)?),
        QueryMsg::DeeplinksByOwnerTime { owner, start_time, end_time, start_after, limit } => 
            to_binary(&query_deeplinks_by_owner_time(deps, env, owner, start_time, end_time, start_after, limit)?),
        QueryMsg::DeeplinksByOwnerTimeAny { owner, start_time, end_time, start_after, limit } => 
            to_binary(&query_deeplinks_by_owner_time_any(deps, env, owner, start_time, end_time, start_after, limit)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    _deps: DepsMut,
    _env: Env,
    _msg: Empty,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}
