#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use cosmwasm_std::{Addr, from_binary, StdError, Uint64};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use serde_json::to_string_pretty;
    use crate::contract::{execute, instantiate, query};
    use crate::msg::*;
    use crate::query::{ConfigResponse, StateResponse};
    use crate::state::{Config, DeeplinkState};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let config: ConfigResponse = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.admins, vec![deps.api.addr_make("admin").to_string()]);
        assert_eq!(config.executors, vec![deps.api.addr_make("executor").to_string()]);
    }

    #[test]
    fn test_create_core_deeplinks() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let file = File::open("core.json").expect("file should open read only");
        let reader = BufReader::new(file);
        let deeplinks: Vec<NamedDeeplink> = serde_json::from_reader(reader).unwrap();

        let mut errors = vec![];
        for deeplink in deeplinks {
            let link = deeplink.clone();
            let msg = ExecuteMsg::CreateNamedDeeplink {
                name: link.id,
                deeplink: Deeplink {
                    type_: link.type_,
                    from: link.from,
                    to: link.to,
                    value: link.value,
                }
            };
            let info = mock_info(deps.api.addr_make("admin").as_str(), &[]);
            let res = execute(deps.as_mut(), mock_env(), info, msg);
            if res.is_err() {
                println!("Deeplink error: {:?} {:?}", res, deeplink);
                errors.push(res);
            } else {
                println!("Deeplink created: {:?}", deeplink);
            }
        }

        assert_eq!(errors.len(), 0);

        let debug_state: StateResponse = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::DebugState {}).unwrap()).unwrap();
        println!("{}", to_string_pretty(&debug_state).unwrap());
    }

    #[test]
    fn test_create_deeplink() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let file = File::open("core.json").expect("file should open read only");
        let reader = BufReader::new(file);
        let deeplinks: Vec<NamedDeeplink> = serde_json::from_reader(reader).unwrap();

        let mut errors = vec![];
        for deeplink in deeplinks {
            let link = deeplink.clone();
            let msg = ExecuteMsg::CreateNamedDeeplink {
                name: link.id,
                deeplink: Deeplink {
                    type_: link.type_,
                    from: link.from,
                    to: link.to,
                    value: link.value,
                }
            };
            let info = mock_info(deps.api.addr_make("admin").as_str(), &[]);
            let res = execute(deps.as_mut(), mock_env(), info, msg);
            if res.is_err() { errors.push(res) };
        }
        assert_eq!(errors.len(), 0);

        let deeplink = Deeplink {
            type_: "Type".to_string(),
            from: Some("Query".to_string()),
            to: Some("String".to_string()),
            value: None,
        };
        let msg = ExecuteMsg::CreateDeeplink { deeplink };
        let info = mock_info("cosmwasm1335hded4gyzpt00fpz75mms4m7ck02wgw07yhw9grahj4dzg4yvqysvwql", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        println!("{:?}", res);
        assert_eq!(res.attributes[0].value, "create_deeplink");

        let last_id: Uint64 = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::LastId {}).unwrap()).unwrap();
        let deeplink_state1: DeeplinkState = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Deeplink { id: last_id }).unwrap()).unwrap();
        assert_eq!(deeplink_state1.type_, "Type");
        assert_eq!(deeplink_state1.from, "Query");
        assert_eq!(deeplink_state1.to, "String");
    }

    #[test]
    fn test_create_deeplink_chat() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let file = File::open("core.json").expect("file should open read only");
        let reader = BufReader::new(file);
        let deeplinks: Vec<NamedDeeplink> = serde_json::from_reader(reader).unwrap();

        let mut errors = vec![];
        for deeplink in deeplinks {
            let link = deeplink.clone();
            let msg = ExecuteMsg::CreateNamedDeeplink {
                name: link.id,
                deeplink: Deeplink {
                    type_: link.type_,
                    from: link.from,
                    to: link.to,
                    value: link.value,
                }
            };
            let info = mock_info(deps.api.addr_make("admin").as_str(), &[]);
            let res = execute(deps.as_mut(), mock_env(), info, msg);
            if res.is_err() { errors.push(res) };
        }
        assert_eq!(errors.len(), 0);

        let file = File::open("project.json").expect("file should open read only");
        let reader = BufReader::new(file);
        let deeplinks: Vec<NamedDeeplink> = serde_json::from_reader(reader).unwrap();

        let mut errors = vec![];
        for deeplink in deeplinks {
            let link = deeplink.clone();
            let msg = ExecuteMsg::CreateNamedDeeplink {
                name: link.id,
                deeplink: Deeplink {
                    type_: link.type_,
                    from: link.from,
                    to: link.to,
                    value: link.value,
                }
            };
            let info = mock_info(deps.api.addr_make("admin").as_str(), &[]);
            let res = execute(deps.as_mut(), mock_env(), info, msg);
            if res.is_err() { errors.push(res) };
        }
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_update_deeplink() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }

    #[test]
    fn test_delete_deeplink() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }
}
