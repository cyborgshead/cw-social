#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::*;
    use crate::query::{ConfigResponse, StateResponse};
    use crate::state::DeeplinkState;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Timestamp, Uint64};
    use serde_json::to_string_pretty;
    use std::fs::File;
    use std::io::BufReader;

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

    #[test]
    fn test_query_deeplinks_by_owner_time() {
        let mut deps = mock_dependencies();
        
        // Setup: Initialize contract
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string(), deps.api.addr_make("test_user").to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        
        // Create test user
        let test_user = deps.api.addr_make("test_user");
        
        // Create timestamps for testing
        let now = mock_env().block.time.nanos();
        let time1 = Timestamp::from_nanos(now);
        let time2 = Timestamp::from_nanos(now + 100_000_000); // 100ms later
        let time3 = Timestamp::from_nanos(now + 200_000_000); // 200ms later
        let time4 = Timestamp::from_nanos(now + 300_000_000); // 300ms later
        
        // Create a mock environment with the first timestamp
        let mut env1 = mock_env();
        env1.block.time = time1;
        
        // Create first deeplink
        let deeplink1 = Deeplink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("First deeplink".to_string()),
        };
        let msg = ExecuteMsg::CreateDeeplink { deeplink: deeplink1 };
        let info = mock_info(test_user.as_str(), &[]);
        let res = execute(deps.as_mut(), env1.clone(), info, msg).unwrap();
        let first_id = res.attributes.iter()
            .find(|attr| attr.key == "id")
            .map(|attr| attr.value.parse::<u64>().unwrap())
            .unwrap();
        
        // Create a mock environment with the second timestamp
        let mut env2 = mock_env();
        env2.block.time = time2;
        
        // Create second deeplink
        let deeplink2 = Deeplink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Second deeplink".to_string()),
        };
        let msg = ExecuteMsg::CreateDeeplink { deeplink: deeplink2 };
        let info = mock_info(test_user.as_str(), &[]);
        let res = execute(deps.as_mut(), env2.clone(), info, msg).unwrap();
        let second_id = res.attributes.iter()
            .find(|attr| attr.key == "id")
            .map(|attr| attr.value.parse::<u64>().unwrap())
            .unwrap();
        
        // Create a mock environment with the third timestamp
        let mut env3 = mock_env();
        env3.block.time = time3;
        
        // Create third deeplink
        let deeplink3 = Deeplink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Third deeplink".to_string()),
        };
        let msg = ExecuteMsg::CreateDeeplink { deeplink: deeplink3 };
        let info = mock_info(test_user.as_str(), &[]);
        let res = execute(deps.as_mut(), env3.clone(), info, msg).unwrap();
        let third_id = res.attributes.iter()
            .find(|attr| attr.key == "id")
            .map(|attr| attr.value.parse::<u64>().unwrap())
            .unwrap();
        
        // Update the first deeplink at time4
        let mut env4 = mock_env();
        env4.block.time = time4;
        
        let update_deeplink = Deeplink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Updated first deeplink".to_string()),
        };
        let msg = ExecuteMsg::UpdateDeeplink { 
            id: first_id,
            deeplink: update_deeplink 
        };
        let info = mock_info(test_user.as_str(), &[]);
        execute(deps.as_mut(), env4.clone(), info, msg).unwrap();
        
        // Test 1: Query all deeplinks by owner (no time filter)
        let query_msg = QueryMsg::DeeplinksByOwner {
            owner: test_user.to_string(),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 3, "Should return all 3 deeplinks");
        
        // Test 2: Query deeplinks by owner with time range (time1 to time2)
        let query_msg = QueryMsg::DeeplinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time2),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 2, "Should return 2 deeplinks created between time1 and time2");
        
        // Test 3: Query deeplinks by owner with time range (time2 to time3)
        let query_msg = QueryMsg::DeeplinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time2,
            end_time: Some(time3),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env2.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 2, "Should return 2 deeplinks created between time2 and time3");
        
        // Test 4: Query deeplinks by owner with time range (time1 to time4)
        let query_msg = QueryMsg::DeeplinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 3, "Should return all 3 deeplinks created between time1 and time4");
        
        // Test 5: Query deeplinks by owner with time range (time3 to time4)
        let query_msg = QueryMsg::DeeplinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 1, "Should return 1 deeplink created between time3 and time4");
        
        // Test 6: Query deeplinks by owner with time_any (time3 to time4)
        let query_msg = QueryMsg::DeeplinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 2, "Should return 2 deeplinks (created or updated) between time3 and time4");
        
        // Find the updated deeplink
        let updated_deeplink = deeplinks.iter().find(|(id, d)| *id == first_id);
        assert!(updated_deeplink.is_some(), "Should include the updated deeplink");
        
        // Test 7: Query with pagination
        let query_msg = QueryMsg::DeeplinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: None,
            limit: Some(2),
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 2, "Should return only 2 deeplinks due to pagination limit");
        
        // Test 8: Query with start_after
        let start_after = deeplinks[0].0; // Use the ID of the first result as start_after
        
        let query_msg = QueryMsg::DeeplinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: Some(start_after),
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 2, "Should return 2 deeplinks after the start_after ID");
    }
    
    #[test]
    fn test_query_deeplinks_by_owner_time_any() {
        let mut deps = mock_dependencies();
        
        // Setup: Initialize contract
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string(),  deps.api.addr_make("test_user").to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        
        // Create test user
        let test_user = deps.api.addr_make("test_user");
        
        // Create timestamps for testing
        let now = mock_env().block.time.nanos();
        let time1 = Timestamp::from_nanos(now);
        let time2 = Timestamp::from_nanos(now + 100_000_000); // 100ms later
        let time3 = Timestamp::from_nanos(now + 200_000_000); // 200ms later
        let time4 = Timestamp::from_nanos(now + 300_000_000); // 300ms later
        
        // Create deeplinks at different times
        let mut env1 = mock_env();
        env1.block.time = time1;
        
        // Create first deeplink at time1
        let deeplink1 = Deeplink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("First deeplink".to_string()),
        };
        let msg = ExecuteMsg::CreateDeeplink { deeplink: deeplink1 };
        let info = mock_info(test_user.as_str(), &[]);
        let res = execute(deps.as_mut(), env1.clone(), info, msg).unwrap();
        let first_id = res.attributes.iter()
            .find(|attr| attr.key == "id")
            .map(|attr| attr.value.parse::<u64>().unwrap())
            .unwrap();
        
        // Create second deeplink at time2
        let mut env2 = mock_env();
        env2.block.time = time2;
        
        let deeplink2 = Deeplink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Second deeplink".to_string()),
        };
        let msg = ExecuteMsg::CreateDeeplink { deeplink: deeplink2 };
        let info = mock_info(test_user.as_str(), &[]);
        execute(deps.as_mut(), env2.clone(), info, msg).unwrap();
        
        // Update first deeplink at time3
        let mut env3 = mock_env();
        env3.block.time = time3;
        
        let update_deeplink = Deeplink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Updated first deeplink".to_string()),
        };
        let msg = ExecuteMsg::UpdateDeeplink { 
            id: first_id,
            deeplink: update_deeplink 
        };
        let info = mock_info(test_user.as_str(), &[]);
        execute(deps.as_mut(), env3.clone(), info, msg).unwrap();
        
        // Create third deeplink at time4
        let mut env4 = mock_env();
        env4.block.time = time4;
        
        let deeplink3 = Deeplink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Third deeplink".to_string()),
        };
        let msg = ExecuteMsg::CreateDeeplink { deeplink: deeplink3 };
        let info = mock_info(test_user.as_str(), &[]);
        execute(deps.as_mut(), env4.clone(), info, msg).unwrap();
        
        // Test 1: Query by creation time only (time1 to time2)
        let query_msg = QueryMsg::DeeplinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time2),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 2, "Should return 2 deeplinks created between time1 and time2");
        
        // Test 2: Query by creation or update time (time1 to time2)
        let query_msg = QueryMsg::DeeplinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time2),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 2, "Should return 2 deeplinks created or updated between time1 and time2");
        
        // Test 3: Query by creation time only (time3 to time4)
        let query_msg = QueryMsg::DeeplinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 1, "Should return 1 deeplink created between time3 and time4");
        
        // Test 4: Query by creation or update time (time3 to time4)
        let query_msg = QueryMsg::DeeplinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 2, "Should return 2 deeplinks created or updated between time3 and time4");
        
        // Check that we have both the updated first deeplink and the third deeplink
        let has_updated = deeplinks.iter().any(|(id, d)| *id == first_id);
        let has_third = deeplinks.iter().any(|(_, d)| d.value == "Third deeplink");
        
        assert!(has_updated, "Should include the updated first deeplink");
        assert!(has_third, "Should include the third deeplink");
        
        // Test 5: Query with pagination
        let query_msg = QueryMsg::DeeplinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: None,
            limit: Some(2),
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(deeplinks.len(), 2, "Should return only 2 deeplinks due to pagination limit");
        
        // Test 6: Query with start_after
        let start_after = deeplinks[0].0; // Use the ID of the first result as start_after
        
        let query_msg = QueryMsg::DeeplinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: Some(start_after),
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let deeplinks: Vec<(u64, DeeplinkState)> = from_binary(&res).unwrap();
        
        assert!(deeplinks.len() > 0, "Should return deeplinks after the start_after ID");
        assert!(deeplinks[0].0 > start_after, "First result ID should be greater than start_after");
    }
}
