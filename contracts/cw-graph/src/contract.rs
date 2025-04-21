#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Addr, Api, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::execute::{execute_create_cyberlink, execute_create_cyberlinks, execute_create_named_cyberlink, execute_delete_cyberlink, execute_update_admins, execute_update_cyberlink, execute_update_executors, execute_create_cyberlink2};
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::query::{query_config, query_cyberlink_by_fid, query_cyberlinks_by_gids, query_cyberlinks_set_by_gids, query_cyberlinks_by_owner, query_cyberlinks_by_owner_time, query_cyberlinks_by_owner_time_any, query_cyberlink_by_gid, query_last_gid, query_cyberlinks_by_fids, query_state, query_cyberlinks_set_by_fids, query_cyberlinks_by_type, query_cyberlinks_by_from, query_cyberlinks_by_to, query_cyberlinks_by_owner_and_type, query_graph_stats};
use crate::semcores::SemanticCore;
use crate::state::{cyberlinks, Config, CyberlinkState, CONFIG, GID, NAMED_CYBERLINKS};

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

    GID.save(deps.storage, &0)?;

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
    let id = GID.load(deps.storage)? + 1;
    GID.save(deps.storage, &id)?;
    cyberlinks().save(deps.storage, id, &CyberlinkState {
        type_: "Type".to_string(),
        from: "Any".to_string(),
        to: "Any".to_string(),
        value: "".to_string(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
        fid: Some("".to_string()),
    })?;
    NAMED_CYBERLINKS.save(deps.storage, "Type", &id)?;

    // Create Any type
    let id = GID.load(deps.storage)? + 1;
    GID.save(deps.storage, &id)?;
    cyberlinks().save(deps.storage, id, &CyberlinkState {
        type_: "Any".to_string(),
        from: "Null".to_string(),
        to: "Null".to_string(),
        value: "".to_string(),
        owner: info.sender.clone(),
        created_at: env.block.time,
        updated_at: None,
        fid: Some("".to_string()),
    })?;
    NAMED_CYBERLINKS.save(deps.storage, "Any", &id)?;
    Ok(())
}

fn load_semantic_core(deps: DepsMut, env: &Env, info: &MessageInfo, core: SemanticCore) -> Result<(), ContractError> {
    let types = core.get_types();
    
    for type_def in types {
        // Skip entries without an ID (already filtered in get_types, but just to be safe)
        if let Some(id_value) = &type_def.id {
            let id = GID.load(deps.storage)? + 1;
            GID.save(deps.storage, &id)?;
            
            let cyberlink_state = CyberlinkState {
                type_: type_def.type_,
                from: type_def.from.unwrap_or_else(|| "Any".to_string()),
                to: type_def.to.unwrap_or_else(|| "Any".to_string()),
                value: type_def.value.map_or_else(String::new, |v| v.to_string()),
                owner: info.sender.clone(),
                created_at: env.block.time,
                updated_at: None,
                fid: Some("".to_string()),
            };

            cyberlinks().save(deps.storage, id, &cyberlink_state)?;
            NAMED_CYBERLINKS.save(deps.storage, id_value, &id)?;
        }
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
        ExecuteMsg::CreateCyberlink2 {
            node_type,
            node_value,
            link_type,
            link_value,
            link_from_existing_id,
            link_to_existing_id,
        } => execute_create_cyberlink2(
            deps, 
            env, 
            info, 
            node_type,
            node_value,
            link_type, 
            link_value, 
            link_from_existing_id, 
            link_to_existing_id
        ),
        ExecuteMsg::CreateCyberlinks { cyberlinks } => execute_create_cyberlinks(deps, env, info, cyberlinks),
        ExecuteMsg::UpdateCyberlink { fid, value } => execute_update_cyberlink(deps, env, info, fid, value),
        ExecuteMsg::DeleteCyberlink { fid } => execute_delete_cyberlink(deps, env, info, fid),
        ExecuteMsg::UpdateAdmins { new_admins } => execute_update_admins(deps, env, info, new_admins),
        ExecuteMsg::UpdateExecutors { new_executors } => execute_update_executors(deps, env, info, new_executors),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        QueryMsg::DebugState {} => to_json_binary(&query_state(deps)?),
        QueryMsg::GetGraphStats { owner, type_ } => to_json_binary(&query_graph_stats(deps, owner, type_)?),
        
        QueryMsg::LastGID {} => to_json_binary(&query_last_gid(deps)?),
        QueryMsg::CyberlinkByGID { gid } => to_json_binary(&query_cyberlink_by_gid(deps, gid)?),
        QueryMsg::CyberlinksByGIDs { start_after_gid, limit} => to_json_binary(&query_cyberlinks_by_gids(deps, start_after_gid, limit)?),
        QueryMsg::CyberlinksSetByGIDs { gids } => to_json_binary(&query_cyberlinks_set_by_gids(deps, gids)?),
        
        QueryMsg::CyberlinkByFID { fid } => to_json_binary(&query_cyberlink_by_fid(deps, fid)?),
        QueryMsg::CyberlinksByFIDs { start_after_fid, limit } => to_json_binary(&query_cyberlinks_by_fids(deps, start_after_fid, limit)?),
        QueryMsg::CyberlinksSetByFIDs { fids } => to_json_binary(&query_cyberlinks_set_by_fids(deps, fids)?),
        
        QueryMsg::CyberlinksByOwner { owner, start_after_gid, limit } => to_json_binary(&query_cyberlinks_by_owner(deps, owner, start_after_gid, limit)?),
        QueryMsg::CyberlinksByOwnerTime { owner, start_time, end_time, start_after_gid, limit } =>
            to_json_binary(&query_cyberlinks_by_owner_time(deps, env, owner, start_time, end_time, start_after_gid, limit)?),
        QueryMsg::CyberlinksByOwnerTimeAny { owner, start_time, end_time, start_after_gid, limit } =>
            to_json_binary(&query_cyberlinks_by_owner_time_any(deps, env, owner, start_time, end_time, start_after_gid, limit)?),

        QueryMsg::CyberlinksByType { type_, start_after_gid, limit } => to_json_binary(&query_cyberlinks_by_type(deps, type_, start_after_gid, limit)?),
        QueryMsg::CyberlinksByFrom { from, start_after_gid, limit } => to_json_binary(&query_cyberlinks_by_from(deps, from, start_after_gid, limit)?),
        QueryMsg::CyberlinksByTo { to, start_after_gid, limit } => to_json_binary(&query_cyberlinks_by_to(deps, to, start_after_gid, limit)?),
        QueryMsg::CyberlinksByOwnerAndType { owner, type_, start_after_gid, limit } => to_json_binary(&query_cyberlinks_by_owner_and_type(deps, owner, type_, start_after_gid, limit)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: Empty,
) -> Result<Response, ContractError> {
    // TODO add get_contract_version and version check later with prod release
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}
