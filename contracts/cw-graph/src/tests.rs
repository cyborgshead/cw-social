#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::msg::*;
    use crate::query::{ConfigResponse, StateResponse};
    use crate::state::CyberlinkState;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Timestamp, Uint64};
    use serde_json::to_string_pretty;
    use std::fs::File;
    use std::io::BufReader;
    use serde::Deserialize;
    use serde_json::Value;

    #[derive(Debug, Clone, Deserialize)]
    struct RawNamedCyberlink {
        id: String,
        #[serde(rename = "type")]
        type_: String,
        from: Option<String>,
        to: Option<String>,
        value: Option<serde_json::Value>,
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let config: ConfigResponse = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.admins, vec![deps.api.addr_make("admin").to_string()]);
        assert_eq!(config.executors, vec![deps.api.addr_make("executor").to_string()]);
    }

    #[test]
    fn test_create_cyberlink_deepcore() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let file = File::open("./semcores/deepcore.json").expect("file should open read only");
        let reader = BufReader::new(file);
        let cyberlinks: Vec<NamedCyberlink> = serde_json::from_reader(reader).unwrap();

        let mut errors = vec![];
        for cyberlink in cyberlinks {
            let link = cyberlink.clone();
            let msg = ExecuteMsg::CreateNamedCyberlink {
                name: link.id,
                cyberlink: Cyberlink {
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
        println!("{:?}", errors);

        let cyberlink = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Query".to_string()),
            to: Some("String".to_string()),
            value: None,
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: cyberlink };
        let info = mock_info("cosmwasm1335hded4gyzpt00fpz75mms4m7ck02wgw07yhw9grahj4dzg4yvqysvwql", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        println!("{:?}", res);
        assert_eq!(res.attributes[0].value, "create_cyberlink");

        let last_id: Uint64 = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::LastId {}).unwrap()).unwrap();
        let cyberlink_state1: CyberlinkState = from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::Cyberlink { id: last_id }).unwrap()).unwrap();
        assert_eq!(cyberlink_state1.type_, "Type");
        assert_eq!(cyberlink_state1.from, "Query");
        assert_eq!(cyberlink_state1.to, "String");
    }

    #[test]
    fn test_create_cyberlink_chat() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let file = File::open("./semcores/chat_example.json").expect("file should open read only");
        let reader = BufReader::new(file);
        let raw_cyberlinks: Vec<RawNamedCyberlink> = serde_json::from_reader(reader).unwrap();

        let cyberlinks: Vec<NamedCyberlink> = raw_cyberlinks
            .into_iter()
            .map(|link| NamedCyberlink {
                id: link.id,
                type_: link.type_,
                from: link.from,
                to: link.to,
                value: link.value.map(|v| serde_json::to_string(&v).unwrap()),
            })
            .collect();

        let mut errors = vec![];
        for cyberlink in cyberlinks {
            let link = cyberlink.clone();
            let msg = ExecuteMsg::CreateNamedCyberlink {
                name: link.id,
                cyberlink: Cyberlink {
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
    fn test_create_cyberlink_project() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let file = File::open("./semcores/project_example.json").expect("file should open read only");
        let reader = BufReader::new(file);
        let cyberlinks: Vec<NamedCyberlink> = serde_json::from_reader(reader).unwrap();

        let mut errors = vec![];
        for cyberlink in cyberlinks {
            let link = cyberlink.clone();
            let msg = ExecuteMsg::CreateNamedCyberlink {
                name: link.id,
                cyberlink: Cyberlink {
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
    fn test_create_cyberlink_social() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let file = File::open("./semcores/social_example.json").expect("file should open read only");
        let reader = BufReader::new(file);
        let raw_cyberlinks: Vec<RawNamedCyberlink> = serde_json::from_reader(reader).unwrap();

        let cyberlinks: Vec<NamedCyberlink> = raw_cyberlinks
            .into_iter()
            .map(|link| NamedCyberlink {
                id: link.id,
                type_: link.type_,
                from: link.from,
                to: link.to,
                value: link.value.map(|v| serde_json::to_string(&v).unwrap()),
            })
            .collect();

        let mut errors = vec![];
        for cyberlink in cyberlinks {
            let link = cyberlink.clone();
            let msg = ExecuteMsg::CreateNamedCyberlink {
                name: link.id,
                cyberlink: Cyberlink {
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
        println!("{:?}", errors);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_create_cyberlink_lens() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let file = File::open("./semcores/lens.json").expect("file should open read only");
        let reader = BufReader::new(file);
        let raw_cyberlinks: Vec<RawNamedCyberlink> = serde_json::from_reader(reader).unwrap();

        let cyberlinks: Vec<NamedCyberlink> = raw_cyberlinks
            .into_iter()
            .map(|link| NamedCyberlink {
                id: link.id,
                type_: link.type_,
                from: link.from,
                to: link.to,
                value: link.value.map(|v| serde_json::to_string(&v).unwrap()),
            })
            .collect();

        let mut errors = vec![];
        for cyberlink in cyberlinks {
            let link = cyberlink.clone();
            let msg = ExecuteMsg::CreateNamedCyberlink {
                name: link.id,
                cyberlink: Cyberlink {
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
        println!("{:?}", errors);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_update_cyberlink() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }

    #[test]
    fn test_delete_cyberlink() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    }

    #[test]
    fn test_query_cyberlinks_by_owner_time() {
        let mut deps = mock_dependencies();
        
        // Setup: Initialize contract
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string(), deps.api.addr_make("test_user").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
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
        
        // Create first cyberlink
        let cyberlink1 = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("First cyberlink".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: cyberlink1 };
        let info = mock_info(test_user.as_str(), &[]);
        let res = execute(deps.as_mut(), env1.clone(), info, msg).unwrap();
        let first_id = res.attributes.iter()
            .find(|attr| attr.key == "id")
            .map(|attr| attr.value.parse::<u64>().unwrap())
            .unwrap();
        
        // Create a mock environment with the second timestamp
        let mut env2 = mock_env();
        env2.block.time = time2;
        
        // Create second cyberlink
        let cyberlink2 = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Second cyberlink".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: cyberlink2 };
        let info = mock_info(test_user.as_str(), &[]);
        let res = execute(deps.as_mut(), env2.clone(), info, msg).unwrap();
        let second_id = res.attributes.iter()
            .find(|attr| attr.key == "id")
            .map(|attr| attr.value.parse::<u64>().unwrap())
            .unwrap();
        
        // Create a mock environment with the third timestamp
        let mut env3 = mock_env();
        env3.block.time = time3;
        
        // Create third cyberlink
        let cyberlink3 = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Third cyberlink".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: cyberlink3 };
        let info = mock_info(test_user.as_str(), &[]);
        let res = execute(deps.as_mut(), env3.clone(), info, msg).unwrap();
        let third_id = res.attributes.iter()
            .find(|attr| attr.key == "id")
            .map(|attr| attr.value.parse::<u64>().unwrap())
            .unwrap();
        
        // Update the first cyberlink at time4
        let mut env4 = mock_env();
        env4.block.time = time4;
        
        let update_cyberlink = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Updated first cyberlink".to_string()),
        };
        let msg = ExecuteMsg::UpdateCyberlink { 
            id: first_id,
            cyberlink: update_cyberlink 
        };
        let info = mock_info(test_user.as_str(), &[]);
        execute(deps.as_mut(), env4.clone(), info, msg).unwrap();
        
        // Test 1: Query all cyberlinks by owner (no time filter)
        let query_msg = QueryMsg::CyberlinksByOwner {
            owner: test_user.to_string(),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 3, "Should return all 3 cyberlinks");
        
        // Test 2: Query cyberlinks by owner with time range (time1 to time2)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time2),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created between time1 and time2");
        
        // Test 3: Query cyberlinks by owner with time range (time2 to time3)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time2,
            end_time: Some(time3),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env2.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created between time2 and time3");
        
        // Test 4: Query cyberlinks by owner with time range (time1 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 3, "Should return all 3 cyberlinks created between time1 and time4");
        
        // Test 5: Query cyberlinks by owner with time range (time3 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 1, "Should return 1 cyberlink created between time3 and time4");
        
        // Test 6: Query cyberlinks by owner with time_any (time3 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks (created or updated) between time3 and time4");
        
        // Find the updated cyberlink
        let updated_cyberlink = cyberlinks.iter().find(|(id, d)| *id == first_id);
        assert!(updated_cyberlink.is_some(), "Should include the updated cyberlink");
        
        // Test 7: Query with pagination
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: None,
            limit: Some(2),
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return only 2 cyberlinks due to pagination limit");
        
        // Test 8: Query with start_after
        let start_after = cyberlinks[0].0; // Use the ID of the first result as start_after
        
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: Some(start_after),
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks after the start_after ID");
    }
    
    #[test]
    fn test_query_cyberlinks_by_owner_time_any() {
        let mut deps = mock_dependencies();
        
        // Setup: Initialize contract
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string(),  deps.api.addr_make("test_user").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
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
        
        // Create cyberlinks at different times
        let mut env1 = mock_env();
        env1.block.time = time1;
        
        // Create first cyberlink at time1
        let cyberlink1 = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("First cyberlink".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: cyberlink1 };
        let info = mock_info(test_user.as_str(), &[]);
        let res = execute(deps.as_mut(), env1.clone(), info, msg).unwrap();
        let first_id = res.attributes.iter()
            .find(|attr| attr.key == "id")
            .map(|attr| attr.value.parse::<u64>().unwrap())
            .unwrap();
        
        // Create second cyberlink at time2
        let mut env2 = mock_env();
        env2.block.time = time2;
        
        let cyberlink2 = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Second cyberlink".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: cyberlink2 };
        let info = mock_info(test_user.as_str(), &[]);
        execute(deps.as_mut(), env2.clone(), info, msg).unwrap();
        
        // Update first cyberlink at time3
        let mut env3 = mock_env();
        env3.block.time = time3;
        
        let update_cyberlink = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Updated first cyberlink".to_string()),
        };
        let msg = ExecuteMsg::UpdateCyberlink { 
            id: first_id,
            cyberlink: update_cyberlink 
        };
        let info = mock_info(test_user.as_str(), &[]);
        execute(deps.as_mut(), env3.clone(), info, msg).unwrap();
        
        // Create third cyberlink at time4
        let mut env4 = mock_env();
        env4.block.time = time4;
        
        let cyberlink3 = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Any".to_string()),
            to: Some("Any".to_string()),
            value: Some("Third cyberlink".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: cyberlink3 };
        let info = mock_info(test_user.as_str(), &[]);
        execute(deps.as_mut(), env4.clone(), info, msg).unwrap();
        
        // Test 1: Query by creation time only (time1 to time2)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time2),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created between time1 and time2");
        
        // Test 2: Query by creation or update time (time1 to time2)
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time2),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created or updated between time1 and time2");
        
        // Test 3: Query by creation time only (time3 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 1, "Should return 1 cyberlink created between time3 and time4");
        
        // Test 4: Query by creation or update time (time3 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created or updated between time3 and time4");
        
        // Check that we have both the updated first cyberlink and the third cyberlink
        let has_updated = cyberlinks.iter().any(|(id, d)| *id == first_id);
        let has_third = cyberlinks.iter().any(|(_, d)| d.value == "Third cyberlink");
        
        assert!(has_updated, "Should include the updated first cyberlink");
        assert!(has_third, "Should include the third cyberlink");
        
        // Test 5: Query with pagination
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: None,
            limit: Some(2),
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return only 2 cyberlinks due to pagination limit");
        
        // Test 6: Query with start_after
        let start_after = cyberlinks[0].0; // Use the ID of the first result as start_after
        
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after: Some(start_after),
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_binary(&res).unwrap();
        assert!(cyberlinks.len() > 0, "Should return cyberlinks after the start_after ID");
        // FIXME
        // assert!(cyberlinks[0].0 > start_after, "First result ID should be greater than start_after");
    }
}
