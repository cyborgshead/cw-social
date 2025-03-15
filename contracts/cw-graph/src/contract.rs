#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, StdResult, MessageInfo, Reply, Api, Addr, Empty, Response, to_json_binary};
use cw2::{get_contract_version, set_contract_version};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG, CyberlinkState, ID, NAMED_CYBERLINKS, cyberlinks};
use crate::execute::{execute_create_cyberlink, execute_delete_cyberlink, execute_update_cyberlink, execute_update_admins, execute_update_executors, execute_create_cyberlinks, execute_create_named_cyberlink};
use crate::query::{query_config, query_cyberlinks, query_cyberlinks_by_ids, query_id, query_last_id, query_named_cyberlinks, query_state, query_cyberlinks_by_owner, query_cyberlinks_by_owner_time, query_cyberlinks_by_owner_time_any};
use semver::Version;
use crate::semcores::{SemanticCore, TypeDefinition};

const CONTRACT_NAME: &str = "crates.io:cw-graph";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
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

    // Initialize base types
    create_base_types(deps.branch(), &env, &info)?;

    // Load selected semantic cores
    for core_name in msg.semantic_cores {
        if let Some(core) = SemanticCore::from_str(&core_name) {
            load_semantic_core(deps.branch(), &env, &info, core)?;
        }
    }

    Ok(Response::default())
}

fn create_base_types(deps: DepsMut, env: &Env, info: &MessageInfo) -> Result<(), ContractError> {
    // Create Type and Any types as before
    let id = ID.load(deps.storage)? + 1;
    ID.save(deps.storage, &id)?;
    cyberlinks().save(deps.storage, id, &CyberlinkState {
        type_: "Type".to_string(),
        from: "Any".to_string(),
        to: "Any".to_string(),
        value: "".to_string(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
    })?;
    NAMED_CYBERLINKS.save(deps.storage,
                          "Type", &CyberlinkState {
            type_: "Type".to_string(),
            from: "Any".to_string(),
            to: "Any".to_string(),
            value: "".to_string(),
            owner: info.sender.clone(),
            created_at: env.block.time,
            updated_at: None,
        })?;

    // Create Any type
    let id = ID.load(deps.storage)? + 1;
    ID.save(deps.storage, &id)?;
    cyberlinks().save(deps.storage, id, &CyberlinkState {
        type_: "Any".to_string(),
        from: "Null".to_string(),
        to: "Null".to_string(),
        value: "".to_string(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
    })?;
    NAMED_CYBERLINKS.save(deps.storage,
                          "Any", &CyberlinkState {
            type_: "Type".to_string(),
            from: "Null".to_string(),
            to: "Null".to_string(),
            value: "".to_string(),
            owner: info.sender.clone(),
            created_at: env.block.time,
            updated_at: None,
        })?;
    
    Ok(())
}

fn load_semantic_core(deps: DepsMut, env: &Env, info: &MessageInfo, core: SemanticCore) -> Result<(), ContractError> {
    let types = core.get_types();
    
    for type_def in types {
        let id = ID.load(deps.storage)? + 1;
        ID.save(deps.storage, &id)?;
        
        let cyberlink_state = CyberlinkState {
            type_: type_def.type_,
            from: type_def.from.unwrap_or_else(|| "Any".to_string()),
            to: type_def.to.unwrap_or_else(|| "Any".to_string()),
            value: type_def.value.map_or_else(String::new, |v| v.to_string()),
            owner: info.sender.clone(),
            created_at: env.block.time,
            updated_at: None,
        };

        cyberlinks().save(deps.storage, id, &cyberlink_state)?;
        NAMED_CYBERLINKS.save(deps.storage, &type_def.id, &cyberlink_state)?;
    }
    
    Ok(())
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
        ExecuteMsg::CreateNamedCyberlink { name, cyberlink } => execute_create_named_cyberlink(deps, env, info, name, cyberlink),
        ExecuteMsg::CreateCyberlink { cyberlink } => execute_create_cyberlink(deps, env, info, cyberlink),
        ExecuteMsg::CreateCyberlinks { cyberlinks } => execute_create_cyberlinks(deps, env, info, cyberlinks),
        ExecuteMsg::UpdateCyberlink { id, cyberlink } => execute_update_cyberlink(deps, env, info, id, cyberlink),
        ExecuteMsg::DeleteCyberlink { id } => execute_delete_cyberlink(deps, env, info, id),
        ExecuteMsg::UpdateAdmins { new_admins } => execute_update_admins(deps, env, info, new_admins),
        ExecuteMsg::UpdateExecutors { new_executors } => execute_update_executors(deps, env, info, new_executors)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::LastId {} => to_json_binary(&query_last_id(deps)?),
        QueryMsg::DebugState {} => to_json_binary(&query_state(deps)?),
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::Cyberlink { id } => to_json_binary(&query_id(deps, id)?),
        QueryMsg::Cyberlinks { start_after, limit} => to_json_binary(&query_cyberlinks(deps, start_after, limit)?),
        QueryMsg::CyberlinksByIds { ids } => to_json_binary(&query_cyberlinks_by_ids(deps, ids)?),
        QueryMsg::NamedCyberlinks { start_after, limit } => to_json_binary(&query_named_cyberlinks(deps, start_after, limit)?),
        QueryMsg::CyberlinksByOwner { owner, start_after, limit } => to_json_binary(&query_cyberlinks_by_owner(deps, owner, start_after, limit)?),
        QueryMsg::CyberlinksByOwnerTime { owner, start_time, end_time, start_after, limit } =>
            to_json_binary(&query_cyberlinks_by_owner_time(deps, env, owner, start_time, end_time, start_after, limit)?),
        QueryMsg::CyberlinksByOwnerTimeAny { owner, start_time, end_time, start_after, limit } =>
            to_json_binary(&query_cyberlinks_by_owner_time_any(deps, env, owner, start_time, end_time, start_after, limit)?),
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
