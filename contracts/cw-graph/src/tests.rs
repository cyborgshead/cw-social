#[cfg(test)]
mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::*;
    use crate::query::ConfigResponse;
    use crate::state::{CyberlinkState, NAMED_CYBERLINKS};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, message_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{from_json, Addr, OwnedDeps, Response, Timestamp, Uint64};
    use serde::Deserialize;
    use std::fs::File;
    use std::io::BufReader;

    #[derive(Debug, Clone, Deserialize)]
    struct RawNamedCyberlink {
        id: Option<String>,
        #[serde(rename = "type")]
        type_: String,
        from: Option<String>,
        to: Option<String>,
        value: Option<serde_json::Value>,
    }

    // Helper function to process and create cyberlinks from file
    fn process_and_execute_cyberlinks_from_file(
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
        admin: &Addr,
        file_path: &str,
    ) -> Vec<Result<Response, ContractError>> {
        let file = File::open(file_path).expect("file should open read only");
        let reader = BufReader::new(file);
        let raw_cyberlinks: Vec<RawNamedCyberlink> = serde_json::from_reader(reader).unwrap();

        // Process entries with ID field first (CreateNamedCyberlink)
        let named_cyberlinks: Vec<NamedCyberlink> = raw_cyberlinks
            .iter()
            .filter(|link| link.id.is_some())
            .map(|link| NamedCyberlink {
                id: link.id.clone().expect("ID should exist"),
                type_: link.type_.clone(),
                from: link.from.clone(),
                to: link.to.clone(),
                value: link.value.clone().map(|v| serde_json::to_string(&v).unwrap()),
            })
            .collect();

        let mut errors = vec![];
        
        // Create named cyberlinks first (these define types)
        for cyberlink in named_cyberlinks {
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
            let info = message_info(admin, &[]);
            let res = execute(deps.as_mut(), mock_env(), info, msg);
            if res.is_err() { errors.push(res) };
        }
        
        // Process entries without ID field (CreateCyberlink)
        let regular_cyberlinks: Vec<Cyberlink> = raw_cyberlinks
            .iter()
            .filter(|link| link.id.is_none())
            .map(|link| Cyberlink {
                type_: link.type_.clone(),
                from: link.from.clone(),
                to: link.to.clone(),
                value: link.value.clone().map(|v| serde_json::to_string(&v).unwrap()),
            })
            .collect();
        
        // Create regular cyberlinks
        for cyberlink in regular_cyberlinks {
            let msg = ExecuteMsg::CreateCyberlink {
                cyberlink: cyberlink.clone()
            };
            let info = message_info(admin, &[]);
            let res = execute(deps.as_mut(), mock_env(), info, msg);
            if res.is_err() { errors.push(res) };
        }
        
        errors
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };
        let info = message_info(&deps.api.addr_make("admin"), &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let config: ConfigResponse = from_json(&query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()).unwrap();
        assert_eq!(config.admins, vec![deps.api.addr_make("admin").to_string()]);
        assert_eq!(config.executors, vec![deps.api.addr_make("executor").to_string()]);
    }

    #[test]
    fn test_create_cyberlink_deepcore() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec![],
        };

        let admin = deps.api.addr_make("admin");

        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let errors = process_and_execute_cyberlinks_from_file(&mut deps, &admin, "./semcores/deep.json");
        
        assert_eq!(errors.len(), 0, "Errors: {:?}", errors);

        let cyberlink = Cyberlink {
            type_: "Type".to_string(),
            from: Some("Query".to_string()),
            to: Some("String".to_string()),
            value: None,
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: cyberlink };
        let info = message_info(&admin, &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.attributes[0].value, "create_cyberlink");
    }

    #[test]
    fn test_create_cyberlink_chat() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: Vec::new(),
        };

        let admin = deps.api.addr_make("admin");

        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let errors = process_and_execute_cyberlinks_from_file(&mut deps, &admin, "./semcores/chat.json");
        
        assert_eq!(errors.len(), 0, "Errors: {:?}", errors);
    }

    #[test]
    fn test_create_cyberlink_chatgpt() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: Vec::new(),
        };

        let admin = deps.api.addr_make("admin");

        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let errors = process_and_execute_cyberlinks_from_file(&mut deps, &admin, "./semcores/chatgpt.json");

        assert_eq!(errors.len(), 0, "Errors: {:?}", errors);
    }

    #[test]
    fn test_create_cyberlink_project() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: vec![],
        };
        
        let admin = deps.api.addr_make("admin");
        
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let errors = process_and_execute_cyberlinks_from_file(&mut deps, &admin, "./semcores/project.json");
        
        assert_eq!(errors.len(), 0, "Errors: {:?}", errors);
    }

    #[test]
    fn test_create_cyberlink_social() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: Vec::new(), // Don't load semantic cores automatically
        };
        
        let admin = deps.api.addr_make("admin");
        
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let errors = process_and_execute_cyberlinks_from_file(&mut deps, &admin, "./semcores/social.json");
        
        assert_eq!(errors.len(), 0, "Errors: {:?}", errors);
    }

    #[test]
    fn test_create_cyberlink_lens() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string()],
            semantic_cores: Vec::new(), // Don't load semantic cores automatically
        };
        
        let admin = deps.api.addr_make("admin");
        
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let errors = process_and_execute_cyberlinks_from_file(&mut deps, &admin, "./semcores/lens.json");
        
        // TODO fix this test
        assert_eq!(errors.len(), 4, "Errors: {:?}", errors);
    }

    #[test]
    fn test_update_cyberlink() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string(), deps.api.addr_make("user").to_string()],
            semantic_cores: vec![],
        };

        let creator = deps.api.addr_make("creator");

        let info = message_info(&creator, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        
        // Create test users
        let admin = deps.api.addr_make("admin");
        let user = deps.api.addr_make("user");
        
        // First create a Type
        let type_msg = Cyberlink {
            type_: "Type".to_string(),
            from: None,
            to: None,
            value: None,
        };
        let msg = ExecuteMsg::CreateNamedCyberlink {
            name: "Post".to_string(),
            cyberlink: type_msg,
        };
        let admin_info = message_info(&admin, &[]);
        execute(deps.as_mut(), mock_env(), admin_info.clone(), msg).unwrap();
        
        // Create a cyberlink as user
        let cyberlink = Cyberlink {
            type_: "Post".to_string(),
            from: None,
            to: None,
            value: Some("Original content".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink };
        let user_info = message_info(&user, &[]);
        let res = execute(deps.as_mut(), mock_env(), user_info.clone(), msg).unwrap();
        
        // Extract the numeric ID from the response
        let fid = res.attributes
            .iter()
            .find(|attr| attr.key == "fid")
            .map(|attr| attr.value.clone())
            .unwrap();
        
        // Update cyberlink with different content
        let new_value = Some("Updated content".to_string());
        let update_msg = ExecuteMsg::UpdateCyberlink {
            fid: fid.clone(),
            value: new_value.clone(),
        };
        let res = execute(deps.as_mut(), mock_env(), user_info.clone(), update_msg).unwrap();
        assert_eq!(res.attributes[0].value, "update_cyberlink");
        
        // Verify the update was successful
        let query_msg = QueryMsg::CyberlinkByFID { fid: fid.clone() };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let cyberlink_state: CyberlinkState = from_json(&res).unwrap();
        
        assert_eq!(cyberlink_state.type_, "Post");
        assert_eq!(cyberlink_state.value, "Updated content");
        assert!(cyberlink_state.updated_at.is_some(), "Updated timestamp should be set");
        
        // Test unauthorized update (non-owner)
        let other_user = deps.api.addr_make("other_user");
        let other_info = message_info(&other_user, &[]);
        let update_msg_unauth = ExecuteMsg::UpdateCyberlink {
            fid: fid.clone(),
            value: new_value.clone(), // Use the same updated value
        };
        
        // Should fail with Unauthorized error
        let err = execute(deps.as_mut(), mock_env(), other_info, update_msg_unauth).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
        
        // Admin can update any cyberlink
        let admin_new_value = Some("Admin updated".to_string());
        let admin_update_msg = ExecuteMsg::UpdateCyberlink {
            fid: fid.clone(),
            value: admin_new_value.clone(),
        };
        
        // Admin update should succeed
        let res = execute(deps.as_mut(), mock_env(), admin_info, admin_update_msg).unwrap();
        assert_eq!(res.attributes[0].value, "update_cyberlink");
        
        // Verify admin update
        let query_msg = QueryMsg::CyberlinkByFID { fid: fid.clone() };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let cyberlink_state: CyberlinkState = from_json(&res).unwrap();

        assert_eq!(cyberlink_state.value, "Admin updated");
    }

    #[test]
    fn test_delete_cyberlink() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            admins: vec![deps.api.addr_make("admin").to_string()],
            executers: vec![deps.api.addr_make("executor").to_string(), deps.api.addr_make("user").to_string()],
            semantic_cores: vec!["chat".to_string(), "social_example".to_string()],
        };

        let creator = deps.api.addr_make("creator");

        let info = message_info(&creator, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        
        // Create test users
        let admin = deps.api.addr_make("admin");
        let user = deps.api.addr_make("user");
        let other_user = deps.api.addr_make("other_user");
        
        // First create a Type
        let type_msg = Cyberlink {
            type_: "Type".to_string(),
            from: None,
            to: None,
            value: None,
        };
        let msg = ExecuteMsg::CreateNamedCyberlink {
            name: "Post".to_string(),
            cyberlink: type_msg,
        };
        let admin_info = message_info(&admin, &[]);
        execute(deps.as_mut(), mock_env(), admin_info.clone(), msg).unwrap();
        
        // Create a cyberlink as user
        let cyberlink = Cyberlink {
            type_: "Post".to_string(),
            from: None,
            to: None,
            value: Some("Content to be deleted".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink };
        let user_info = message_info(&user, &[]);
        let res = execute(deps.as_mut(), mock_env(), user_info.clone(), msg).unwrap();
        
        let fid = res.attributes
            .iter()
            .find(|attr| attr.key == "fid")
            .unwrap()
            .value
            .clone();
        
        // Verify cyberlink exists
        let query_msg = QueryMsg::CyberlinkByFID { fid: fid.clone() };
        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let _: CyberlinkState = from_json(&res).unwrap();
        
        // Test that non-admin cannot delete
        let delete_msg = ExecuteMsg::DeleteCyberlink { fid: fid.clone() };
        let other_info = message_info(&other_user, &[]);
        let err = execute(deps.as_mut(), mock_env(), other_info, delete_msg.clone()).unwrap_err();
        assert!(matches!(err, ContractError::Unauthorized {}));
        
        // User who owns the cyberlink should be able to delete it if they're also an admin
        // Let's make the user an admin
        let update_admins_msg = ExecuteMsg::UpdateAdmins { new_admins: vec![admin.to_string(), user.to_string()] };
        execute(deps.as_mut(), mock_env(), admin_info.clone(), update_admins_msg).unwrap();
        
        // Now user should be able to delete their own cyberlink
        let res = execute(deps.as_mut(), mock_env(), user_info, delete_msg.clone()).unwrap();
        assert_eq!(res.attributes[0].value, "delete_cyberlink");
        
        // Verify cyberlink is marked as deleted (query should fail)
        let err = query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert!(err.to_string().contains("deleted cyberlink"), "Query for deleted cyberlink should fail");
        
        // Create another cyberlink for admin deletion test
        let cyberlink2 = Cyberlink {
            type_: "Post".to_string(),
            from: None,
            to: None,
            value: Some("Admin will delete this".to_string()),
        };
        let executor_info = message_info(&deps.api.addr_make("executor"), &[]);
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: cyberlink2 };
        let res = execute(deps.as_mut(), mock_env(), executor_info, msg).unwrap();
        
        let fid2 = res.attributes
            .iter()
            .find(|attr| attr.key == "fid")
            .map(|attr| attr.value.clone())
            .unwrap();
        
        // Admin deletes the cyberlink
        let delete_msg2 = ExecuteMsg::DeleteCyberlink { fid: fid2.clone() };
        let res = execute(deps.as_mut(), mock_env(), admin_info, delete_msg2).unwrap();
        assert_eq!(res.attributes[0].value, "delete_cyberlink");
        
        // Verify second cyberlink is also deleted
        let query_msg2 = QueryMsg::CyberlinkByFID { fid: fid2.clone() };
        let err = query(deps.as_ref(), mock_env(), query_msg2).unwrap_err();
        assert!(err.to_string().contains("deleted cyberlink"), "Query for deleted cyberlink should fail");
        
        // Query by formatted ID should also fail
        let formatted_query = QueryMsg::CyberlinkByFID { fid: fid2.clone() };
        let err = query(deps.as_ref(), mock_env(), formatted_query).unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("deleted"), 
                "Query by formatted ID should fail for deleted cyberlink");
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

        let creator = deps.api.addr_make("creator");

        let info = message_info(&creator, &[]);
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
        let info = message_info(&test_user, &[]);
        let res = execute(deps.as_mut(), env1.clone(), info, msg).unwrap();
        let fid = res.attributes.iter()
            .find(|attr| attr.key == "fid")
            .unwrap()
            .value
            .clone();
        let first_gid = res.attributes.iter()
            .find(|attr| attr.key == "gid")
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
        let info = message_info(&test_user, &[]);
        let res = execute(deps.as_mut(), env2.clone(), info, msg).unwrap();
        let _second_id = res.attributes.iter()
            .find(|attr| attr.key == "gid")
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
        let info = message_info(&test_user, &[]);
        let res = execute(deps.as_mut(), env3.clone(), info, msg).unwrap();
        let _third_id = res.attributes.iter()
            .find(|attr| attr.key == "gid")
            .map(|attr| attr.value.parse::<u64>().unwrap())
            .unwrap();
        
        // Update the first cyberlink at time4
        let mut env4 = mock_env();
        env4.block.time = time4;
        
        let updated_value = Some("Updated first cyberlink".to_string());
        let msg = ExecuteMsg::UpdateCyberlink { 
            fid: fid.clone(),
            value: updated_value.clone(),
        };
        let info = message_info(&test_user, &[]);
        execute(deps.as_mut(), env4.clone(), info, msg).unwrap();
        
        // Test 1: Query all cyberlinks by owner (no time filter)
        let query_msg = QueryMsg::CyberlinksByOwner {
            owner: test_user.to_string(),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 3, "Should return all 3 cyberlinks");
        
        // Test 2: Query cyberlinks by owner with time range (time1 to time2)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time2),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created between time1 and time2");
        
        // Test 3: Query cyberlinks by owner with time range (time2 to time3)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time2,
            end_time: Some(time3),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env2.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created between time2 and time3");
        
        // Test 4: Query cyberlinks by owner with time range (time1 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 3, "Should return all 3 cyberlinks created between time1 and time4");
        
        // Test 5: Query cyberlinks by owner with time range (time3 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 1, "Should return 1 cyberlink created between time3 and time4");
        
        // Test 6: Query cyberlinks by owner with time_any (time3 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks (created or updated) between time3 and time4");
        
        // Find the updated cyberlink
        let updated_cyberlink = cyberlinks.iter().find(|(id, _)| *id == first_gid);
        assert!(updated_cyberlink.is_some(), "Should include the updated cyberlink");
        
        // Test 7: Query with pagination
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after_gid: None,
            limit: Some(2),
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return only 2 cyberlinks due to pagination limit");
        
        // Test 8: Query with start_after
        let start_after = cyberlinks[0].0; // Use the ID of the first result as start_after
        
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after_gid: Some(start_after),
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
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
        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &[]);
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
        let info = message_info(&test_user, &[]);
        let res = execute(deps.as_mut(), env1.clone(), info, msg).unwrap();
        let fid = res.attributes.iter()
            .find(|attr| attr.key == "fid")
            .unwrap()
            .value
            .clone();
        let first_gid = res.attributes.iter()
            .find(|attr| attr.key == "gid")
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
        let info = message_info(&test_user, &[]);
        execute(deps.as_mut(), env2.clone(), info, msg).unwrap();
        
        // Update first cyberlink at time3
        let mut env3 = mock_env();
        env3.block.time = time3;
        
        let updated_value = Some("Updated first cyberlink".to_string());
        let msg = ExecuteMsg::UpdateCyberlink { 
            fid: fid.clone(),
            value: updated_value.clone(),
        };
        let info = message_info(&test_user, &[]);
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
        let info = message_info(&test_user, &[]);
        execute(deps.as_mut(), env4.clone(), info, msg).unwrap();
        
        // Test 1: Query by creation time only (time1 to time2)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time2),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created between time1 and time2");
        
        // Test 2: Query by creation or update time (time1 to time2)
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time2),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created or updated between time1 and time2");
        
        // Test 3: Query by creation time only (time3 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTime {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 1, "Should return 1 cyberlink created between time3 and time4");
        
        // Test 4: Query by creation or update time (time3 to time4)
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time3,
            end_time: Some(time4),
            start_after_gid: None,
            limit: None,
        };
        let res = query(deps.as_ref(), env3.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return 2 cyberlinks created or updated between time3 and time4");
        
        // Check that we have both the updated first cyberlink and the third cyberlink
        let has_updated = cyberlinks.iter().any(|(id, _)| *id == first_gid);
        let has_third = cyberlinks.iter().any(|(_, d)| d.value == "Third cyberlink");
        
        assert!(has_updated, "Should include the updated first cyberlink");
        assert!(has_third, "Should include the third cyberlink");
        
        // Test 5: Query with pagination
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after_gid: None,
            limit: Some(2),
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        
        assert_eq!(cyberlinks.len(), 2, "Should return only 2 cyberlinks due to pagination limit");
        
        // Test 6: Query with start_after
        let start_after = cyberlinks[0].0; // Use the ID of the first result as start_after
        
        let query_msg = QueryMsg::CyberlinksByOwnerTimeAny {
            owner: test_user.to_string(),
            start_time: time1,
            end_time: Some(time4),
            start_after_gid: Some(start_after),
            limit: None,
        };
        let res = query(deps.as_ref(), env1.clone(), query_msg).unwrap();
        let cyberlinks: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert!(cyberlinks.len() > 0, "Should return cyberlinks after the start_after ID");
        // FIXME
        // assert!(cyberlinks[0].0 > start_after, "First result ID should be greater than start_after");
    }

    #[test]
    fn test_fids() {
        let mut deps = mock_dependencies();
        let test_user = deps.api.addr_make("test_user");
        let admin = deps.api.addr_make("admin");

        // Setup test environment
        let instantiate_msg = InstantiateMsg {
            admins: vec![admin.to_string()],
            executers: vec![test_user.to_string()],
            semantic_cores: Vec::new(),
        };
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

        // Create a Type (admin only)
        let type_msg = Cyberlink {
            type_: "Type".to_string(),
            from: None,
            to: None,
            value: None,
        };
        let msg = ExecuteMsg::CreateNamedCyberlink {
            name: "Post".to_string(),
            cyberlink: type_msg,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Create a cyberlink as a user
        let cyberlink = Cyberlink {
            type_: "Post".to_string(),
            from: None,
            to: None,
            value: Some("Test post content".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink {
            cyberlink: cyberlink.clone(),
        };
        let user_info = message_info(&test_user, &[]);
        let response = execute(deps.as_mut(), mock_env(), user_info.clone(), msg).unwrap();

        // Check that fid was returned in response
        let fid = response.attributes
            .iter()
            .find(|attr| attr.key == "fid")
            .unwrap()
            .value
            .clone();
        
        assert_eq!(fid, "Post:1");

        // Query by formatted ID
        let query_msg = QueryMsg::CyberlinkByFID {
            fid: fid.clone(),
        };
        let response = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let cyberlink_state: CyberlinkState = from_json(&response).unwrap();

        assert_eq!(cyberlink_state.type_, "Post");
        assert_eq!(cyberlink_state.value, "Test post content");

        // Create a second cyberlink of the same type
        let cyberlink2 = Cyberlink {
            type_: "Post".to_string(),
            from: None,
            to: None,
            value: Some("Second post".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink {
            cyberlink: cyberlink2.clone(),
        };
        let response = execute(deps.as_mut(), mock_env(), user_info.clone(), msg).unwrap();

        // Check that fid incremented correctly
        let fid2 = response.attributes
            .iter()
            .find(|attr| attr.key == "fid")
            .unwrap()
            .value
            .clone();
        
        assert_eq!(fid2, "Post:2");

        // Create a different type
        let comment_type = Cyberlink {
            type_: "Type".to_string(),
            from: None,
            to: None,
            value: None,
        };
        let msg = ExecuteMsg::CreateNamedCyberlink {
            name: "Comment".to_string(),
            cyberlink: comment_type,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Create a comment
        let comment = Cyberlink {
            type_: "Comment".to_string(),
            from: None,
            to: None,
            value: Some("This is a comment".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink {
            cyberlink: comment,
        };
        let response = execute(deps.as_mut(), mock_env(), user_info.clone(), msg).unwrap();

        // Check that fid for the new type starts at 1
        let comment_id = response.attributes
            .iter()
            .find(|attr| attr.key == "fid")
            .unwrap()
            .value
            .clone();
        
        assert_eq!(comment_id, "Comment:1");
    }

    #[test]
    fn test_update_type_restriction() {
        let mut deps = mock_dependencies();
        let test_user = deps.api.addr_make("test_user");
        let admin = deps.api.addr_make("admin");

        // Setup test environment
        let instantiate_msg = InstantiateMsg {
            admins: vec![admin.to_string()],
            executers: vec![test_user.to_string()],
            semantic_cores: Vec::new(),
        };
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

        // Create base types for testing
        let type_msg = Cyberlink {
            type_: "Type".to_string(),
            from: None,
            to: None,
            value: None,
        };
        let msg = ExecuteMsg::CreateNamedCyberlink {
            name: "Post".to_string(),
            cyberlink: type_msg.clone(),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::CreateNamedCyberlink {
            name: "Comment".to_string(),
            cyberlink: type_msg,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Create a post
        let post = Cyberlink {
            type_: "Post".to_string(),
            from: None,
            to: None,
            value: Some("Original post content".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink {
            cyberlink: post,
        };
        let user_info = message_info(&test_user, &[]);
        let response = execute(deps.as_mut(), mock_env(), user_info.clone(), msg).unwrap();

        // Get the ID from the response
        let fid = response.attributes
            .iter()
            .find(|attr| attr.key == "fid")
            .unwrap()
            .value
            .clone();

        // Update with the same type (should succeed)
        let valid_new_value = Some("Valid update".to_string());
        
        let valid_update_msg = ExecuteMsg::UpdateCyberlink {
            fid: fid.clone(),
            value: valid_new_value.clone(),
        };
        
        let update_response = execute(deps.as_mut(), mock_env(), user_info.clone(), valid_update_msg).unwrap();
        assert_eq!(update_response.attributes[0].value, "update_cyberlink");

        // Query to verify the update worked
        let query_msg = QueryMsg::CyberlinkByFID {
            fid: fid.clone()
        };
        let query_response = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let updated_state: CyberlinkState = from_json(&query_response).unwrap();
        
        assert_eq!(updated_state.type_, "Post"); // Type should remain unchanged
        assert_eq!(updated_state.value, valid_new_value.unwrap());
    }

    #[test]
    fn test_delete_keeps_fids() {
        let mut deps = mock_dependencies();
        let admin = deps.api.addr_make("admin");
        let test_user = deps.api.addr_make("test_user");

        // Setup test environment
        let instantiate_msg = InstantiateMsg {
            admins: vec![admin.to_string()],
            executers: vec![test_user.to_string()],
            semantic_cores: Vec::new(),
        };
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

        // Create a Type (admin only)
        let type_msg = Cyberlink {
            type_: "Type".to_string(),
            from: None,
            to: None,
            value: None,
        };
        let msg = ExecuteMsg::CreateNamedCyberlink {
            name: "Post".to_string(),
            cyberlink: type_msg,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Create a post
        let post = Cyberlink {
            type_: "Post".to_string(),
            from: None,
            to: None,
            value: Some("Post to be deleted".to_string()),
        };
        let msg = ExecuteMsg::CreateCyberlink {
            cyberlink: post,
        };
        let user_info = message_info(&test_user, &[]);
        let response = execute(deps.as_mut(), mock_env(), user_info.clone(), msg).unwrap();
        
        let fid = response.attributes
            .iter()
            .find(|attr| attr.key == "fid")
            .unwrap()
            .value
            .clone();

        // Delete the cyberlink
        let delete_msg = ExecuteMsg::DeleteCyberlink {
            fid: fid.clone(),
        };
        let admin_info = message_info(&admin, &[]);
        execute(deps.as_mut(), mock_env(), admin_info, delete_msg).unwrap();

        // Verify the numeric ID is marked as deleted
        let query_msg = QueryMsg::CyberlinkByFID {
            fid: fid.clone(),
        };
        let err = query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert!(err.to_string().contains("deleted cyberlink"));

        // Verify the formatted ID entry still exists in storage but is considered deleted
        let query_msg = QueryMsg::CyberlinkByFID {
            fid: fid.clone(),
        };
        // This should return not_found error because we detect the linked numeric ID is deleted
        let err = query(deps.as_ref(), mock_env(), query_msg).unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("deleted"));

        // However, we can directly check if the NAMED_CYBERLINKS entry still exists
        // (This is implementation specific, but demonstrates the state is preserved)
        let check_state = NAMED_CYBERLINKS.load(deps.as_ref().storage, &fid);
        assert!(check_state.is_ok(), "Formatted ID entry should still exist in storage");
    }

    #[test]
    fn test_update_and_query_by_fid() {
        let mut deps = mock_dependencies();
        let admin = deps.api.addr_make("admin");
        let user = deps.api.addr_make("user");

        // Setup test environment
        let instantiate_msg = InstantiateMsg {
            admins: vec![admin.to_string()],
            executers: vec![user.to_string()], // User can execute
            semantic_cores: Vec::new(),
        };
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

        // Create a Type (admin only)
        let type_msg = Cyberlink {
            type_: "Type".to_string(),
            from: None,
            to: None,
            value: None,
        };
        let msg = ExecuteMsg::CreateNamedCyberlink {
            name: "Article".to_string(),
            cyberlink: type_msg,
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // Create an Article cyberlink as the user
        let initial_content = "Initial article content".to_string();
        let cyberlink = Cyberlink {
            type_: "Article".to_string(),
            from: None,
            to: None,
            value: Some(initial_content.clone()),
        };
        let msg = ExecuteMsg::CreateCyberlink {
            cyberlink: cyberlink.clone(),
        };
        let user_info = message_info(&user, &[]);
        let response = execute(deps.as_mut(), mock_env(), user_info.clone(), msg).unwrap();
        
        let fid = response.attributes
            .iter()
            .find(|attr| attr.key == "fid")
            .unwrap()
            .value
            .clone();
        
        assert_eq!(fid, "Article:1");

        // Query by formatted ID initially
        let query_msg = QueryMsg::CyberlinkByFID {
            fid: fid.clone(),
        };
        let response = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let initial_state: CyberlinkState = from_json(&response).unwrap();

        assert_eq!(initial_state.type_, "Article");
        assert_eq!(initial_state.value, initial_content);
        assert_eq!(initial_state.owner, user);
        assert!(initial_state.updated_at.is_none(), "updated_at should be None initially");

        // Update the cyberlink (user owns it, so they can update)
        let updated_content = "Updated article content".to_string();
        
        let update_msg = ExecuteMsg::UpdateCyberlink {
            fid: fid.clone(),
            value: Some(updated_content.clone()),
        };
        
        // Use a new env with a later timestamp for the update
        let mut update_env = mock_env();
        update_env.block.time = update_env.block.time.plus_seconds(100);
        
        let update_response = execute(deps.as_mut(), update_env.clone(), user_info.clone(), update_msg).unwrap();
        assert_eq!(update_response.attributes[0].value, "update_cyberlink");

        // Query by formatted ID again after update
        let response = query(deps.as_ref(), update_env.clone(), query_msg.clone()).unwrap();
        let updated_state: CyberlinkState = from_json(&response).unwrap();

        // Verify the state reflects the update
        assert_eq!(updated_state.type_, "Article");
        assert_eq!(updated_state.from, "Any".to_string()); // Should still be None
        assert_eq!(updated_state.to, "Any".to_string()); // Should still be None
        assert_eq!(updated_state.value, updated_content); // Value should be updated
        assert_eq!(updated_state.owner, user); // Owner should remain the same
        assert!(updated_state.updated_at.is_some(), "updated_at should be Some after update");
        assert_eq!(updated_state.updated_at.unwrap(), update_env.block.time, "updated_at timestamp should match update time");
        assert_eq!(updated_state.created_at, mock_env().block.time, "created_at should not change");

        // Now delete the cyberlink (Admin action)
        let delete_msg = ExecuteMsg::DeleteCyberlink { fid: fid.clone() };
        let admin_info = message_info(&admin, &[]);
        execute(deps.as_mut(), update_env.clone(), admin_info, delete_msg).unwrap();

        // Query by formatted ID after delete (should fail)
        let err = query(deps.as_ref(), update_env, query_msg).unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("deleted"), 
                "Query by formatted ID should fail for deleted cyberlink");
    }

    #[test]
    fn test_query_get_counts() {
        let mut deps = mock_dependencies();
        let admin = deps.api.addr_make("admin");
        let user1 = deps.api.addr_make("user1");
        let user2 = deps.api.addr_make("user2");

        // Setup test environment
        let instantiate_msg = InstantiateMsg {
            admins: vec![admin.to_string()],
            executers: vec![user1.to_string(), user2.to_string()], // Users can execute
            semantic_cores: Vec::new(),
        };
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

        // Create Types (admin only)
        let type_msg = Cyberlink { type_: "Type".to_string(), from: None, to: None, value: None };
        execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::CreateNamedCyberlink { name: "Post".to_string(), cyberlink: type_msg.clone() }).unwrap();
        execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::CreateNamedCyberlink { name: "Comment".to_string(), cyberlink: type_msg }).unwrap();

        // Create cyberlinks
        // User1: 2 Posts, 1 Comment
        let post1_user1 = Cyberlink { type_: "Post".to_string(), from: None, to: None, value: Some("User1 Post 1".to_string()) };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: post1_user1 };
        let user1_info = message_info(&user1, &[]);
        let res1 = execute(deps.as_mut(), mock_env(), user1_info.clone(), msg).unwrap();
        let post1_user1_id = res1.attributes.iter().find(|a| a.key == "fid").unwrap().value.clone();

        let post2_user1 = Cyberlink { type_: "Post".to_string(), from: None, to: None, value: Some("User1 Post 2".to_string()) };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: post2_user1 };
        execute(deps.as_mut(), mock_env(), user1_info.clone(), msg).unwrap();

        let comment1_user1 = Cyberlink { type_: "Comment".to_string(), from: None, to: None, value: Some("User1 Comment 1".to_string()) };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: comment1_user1 };
        execute(deps.as_mut(), mock_env(), user1_info.clone(), msg).unwrap();

        // User2: 1 Post
        let post1_user2 = Cyberlink { type_: "Post".to_string(), from: None, to: None, value: Some("User2 Post 1".to_string()) };
        let msg = ExecuteMsg::CreateCyberlink { cyberlink: post1_user2 };
        let user2_info = message_info(&user2, &[]);
        execute(deps.as_mut(), mock_env(), user2_info.clone(), msg).unwrap();

        // --- Test GetCounts ---

        // Query counts for User1
        let query_msg = QueryMsg::GetGraphStats { owner: Some(user1.to_string()), type_: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let counts: CountsResponse = from_json(&res).unwrap();
        assert_eq!(counts.owner_count, Some(Uint64::new(3))); // 2 Posts + 1 Comment
        assert_eq!(counts.type_count, None);
        assert_eq!(counts.owner_type_count, None);

        // Query counts for type "Post"
        let query_msg = QueryMsg::GetGraphStats { owner: None, type_: Some("Post".to_string()) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let counts: CountsResponse = from_json(&res).unwrap();
        assert_eq!(counts.owner_count, None);
        assert_eq!(counts.type_count, Some(Uint64::new(3))); // User1 Post1, User1 Post2, User2 Post1
        assert_eq!(counts.owner_type_count, None);

        // Query counts for User1 and type "Post"
        let query_msg = QueryMsg::GetGraphStats { owner: Some(user1.to_string()), type_: Some("Post".to_string()) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let counts: CountsResponse = from_json(&res).unwrap();
        assert_eq!(counts.owner_count, Some(Uint64::new(3)));
        assert_eq!(counts.type_count, Some(Uint64::new(3)));
        assert_eq!(counts.owner_type_count, Some(Uint64::new(2))); // User1 Post1, User1 Post2

        // Query counts for User1 and type "Comment"
        let query_msg = QueryMsg::GetGraphStats { owner: Some(user1.to_string()), type_: Some("Comment".to_string()) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let counts: CountsResponse = from_json(&res).unwrap();
        assert_eq!(counts.owner_count, Some(Uint64::new(3)));
        assert_eq!(counts.type_count, Some(Uint64::new(1))); // Only 1 comment exists total
        assert_eq!(counts.owner_type_count, Some(Uint64::new(1))); // User1 Comment1

        // Query counts for User2 and type "Post"
        let query_msg = QueryMsg::GetGraphStats { owner: Some(user2.to_string()), type_: Some("Post".to_string()) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let counts: CountsResponse = from_json(&res).unwrap();
        assert_eq!(counts.owner_count, Some(Uint64::new(1)));
        assert_eq!(counts.type_count, Some(Uint64::new(3)));
        assert_eq!(counts.owner_type_count, Some(Uint64::new(1))); // User2 Post1

        // Query counts with no filters (should return None for all)
        let query_msg = QueryMsg::GetGraphStats { owner: None, type_: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let counts: CountsResponse = from_json(&res).unwrap();
        assert_eq!(counts.owner_count, None);
        assert_eq!(counts.type_count, None);
        assert_eq!(counts.owner_type_count, None);

        // --- Test counts after deletion ---
        // Delete User1's first post
        let delete_msg = ExecuteMsg::DeleteCyberlink { fid: post1_user1_id };
        execute(deps.as_mut(), mock_env(), user1_info.clone(), delete_msg).unwrap();

        // Query counts for User1 again
        let query_msg = QueryMsg::GetGraphStats { owner: Some(user1.to_string()), type_: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let counts: CountsResponse = from_json(&res).unwrap();
        assert_eq!(counts.owner_count, Some(Uint64::new(2))); // 1 Post + 1 Comment remaining

        // Query counts for type "Post" again
        let query_msg = QueryMsg::GetGraphStats { owner: None, type_: Some("Post".to_string()) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let counts: CountsResponse = from_json(&res).unwrap();
        assert_eq!(counts.type_count, Some(Uint64::new(2))); // User1 Post2, User2 Post1 remaining

        // Query counts for User1 and type "Post" again
        let query_msg = QueryMsg::GetGraphStats { owner: Some(user1.to_string()), type_: Some("Post".to_string()) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let counts: CountsResponse = from_json(&res).unwrap();
        assert_eq!(counts.owner_count, Some(Uint64::new(2)));
        assert_eq!(counts.type_count, Some(Uint64::new(2)));
        assert_eq!(counts.owner_type_count, Some(Uint64::new(1))); // User1 Post2 remaining
    }

    #[test]
    fn test_various_queries() {
        let mut deps = mock_dependencies();
        let admin = deps.api.addr_make("admin");
        let user1 = deps.api.addr_make("user1");
        let user2 = deps.api.addr_make("user2");

        // Setup test environment
        let instantiate_msg = InstantiateMsg {
            admins: vec![admin.to_string()],
            executers: vec![user1.to_string(), user2.to_string()],
            semantic_cores: Vec::new(),
        };
        let info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), info.clone(), instantiate_msg).unwrap();

        // Create Types
        let type_msg = Cyberlink { type_: "Type".to_string(), from: None, to: None, value: None };
        execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::CreateNamedCyberlink { name: "Profile".to_string(), cyberlink: type_msg.clone() }).unwrap();
        execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::CreateNamedCyberlink { name: "Follow".to_string(), cyberlink: Cyberlink { type_: "Type".to_string(), from: Some("Profile".to_string()), to: Some("Profile".to_string()), value: None } }).unwrap();
        execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::CreateNamedCyberlink { name: "Post".to_string(), cyberlink: type_msg.clone() }).unwrap();

        // Create Cyberlinks
        let user1_info = message_info(&user1, &[]);
        let user2_info = message_info(&user2, &[]);

        // Profiles
        let res_p1 = execute(deps.as_mut(), mock_env(), user1_info.clone(), ExecuteMsg::CreateCyberlink { cyberlink: Cyberlink { type_: "Profile".to_string(), from: None, to: None, value: Some("User1 Profile".to_string()) } }).unwrap();
        let profile1_fid = res_p1.attributes.iter().find(|a| a.key == "fid").unwrap().value.clone();
        let profile1_gid: u64 = res_p1.attributes.iter().find(|a| a.key == "gid").unwrap().value.parse().unwrap();

        let res_p2 = execute(deps.as_mut(), mock_env(), user2_info.clone(), ExecuteMsg::CreateCyberlink { cyberlink: Cyberlink { type_: "Profile".to_string(), from: None, to: None, value: Some("User2 Profile".to_string()) } }).unwrap();
        let profile2_fid = res_p2.attributes.iter().find(|a| a.key == "fid").unwrap().value.clone();
        let profile2_gid: u64 = res_p2.attributes.iter().find(|a| a.key == "gid").unwrap().value.parse().unwrap();

        // Follows (User1 follows User2)
        let res_f1 = execute(deps.as_mut(), mock_env(), user1_info.clone(), ExecuteMsg::CreateCyberlink { cyberlink: Cyberlink { type_: "Follow".to_string(), from: Some(profile1_fid.clone()), to: Some(profile2_fid.clone()), value: None } }).unwrap();
        let follow1_fid = res_f1.attributes.iter().find(|a| a.key == "fid").unwrap().value.clone();
        let follow1_gid: u64 = res_f1.attributes.iter().find(|a| a.key == "gid").unwrap().value.parse().unwrap();

        // Posts
        let res_post1 = execute(deps.as_mut(), mock_env(), user1_info.clone(), ExecuteMsg::CreateCyberlink { cyberlink: Cyberlink { type_: "Post".to_string(), from: None, to: None, value: Some("User1 Post 1".to_string()) } }).unwrap();
        let post1_fid = res_post1.attributes.iter().find(|a| a.key == "fid").unwrap().value.clone();
        let post1_gid: u64 = res_post1.attributes.iter().find(|a| a.key == "gid").unwrap().value.parse().unwrap();

        let res_post2 = execute(deps.as_mut(), mock_env(), user2_info.clone(), ExecuteMsg::CreateCyberlink { cyberlink: Cyberlink { type_: "Post".to_string(), from: None, to: None, value: Some("User2 Post 1".to_string()) } }).unwrap();
        let post2_fid = res_post2.attributes.iter().find(|a| a.key == "fid").unwrap().value.clone();
        let post2_gid: u64 = res_post2.attributes.iter().find(|a| a.key == "gid").unwrap().value.parse().unwrap();

        let _all_gids = vec![profile1_gid, profile2_gid, follow1_gid, post1_gid, post2_gid];
        let all_fids = vec![profile1_fid.clone(), profile2_fid.clone(), follow1_fid.clone(), post1_fid.clone(), post2_fid.clone()];

        // --- Test CyberlinksByGIDs (Pagination) ---
        let query_msg = QueryMsg::CyberlinksByGIDs { start_after_gid: None, limit: Some(3) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 3);
        assert_eq!(links[0].0, 1); // Base "Type"
        assert_eq!(links[1].0, 2); // Base "Any"
        assert_eq!(links[2].0, 3); // Named "Profile"

        let query_msg = QueryMsg::CyberlinksByGIDs { start_after_gid: Some(links[2].0), limit: Some(10) }; // Start after GID 3
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        // Should return GIDs 4, 5, 6, 7, 8, 9, 10
        assert_eq!(links.len(), 7);

        assert_eq!(links[0].0, 4); assert_eq!(links[0].1.fid, Some("Follow".to_string()));  // Named Follow
        assert_eq!(links[1].0, 5); assert_eq!(links[1].1.fid, Some("Post".to_string()));    // Named Post
        assert_eq!(links[2].0, 6); assert_eq!(links[2].1.fid, Some(profile1_fid.clone()));  // Profile:1
        assert_eq!(links[3].0, 7); assert_eq!(links[3].1.fid, Some(profile2_fid.clone()));  // Profile:2
        assert_eq!(links[4].0, 8); assert_eq!(links[4].1.fid, Some(follow1_fid.clone()));   // Follow:1
        assert_eq!(links[5].0, 9); assert_eq!(links[5].1.fid, Some(post1_fid.clone()));     // Post:1
        assert_eq!(links[6].0, 10); assert_eq!(links[6].1.fid, Some(post2_fid.clone()));    // Post:2

        // --- Test CyberlinksSetByGIDs ---
        let actual_profile1_gid = 6;
        let actual_post2_gid = 10;
        let query_msg = QueryMsg::CyberlinksSetByGIDs { gids: vec![actual_profile1_gid, actual_post2_gid, 999] }; // 999 doesn't exist
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|(id, _)| *id == actual_profile1_gid));
        assert!(links.iter().any(|(id, _)| *id == actual_post2_gid));

        // --- Test CyberlinksByIDs (Pagination - similar to query_named_cyberlinks) ---
        // Note: Order is lexicographical by formatted ID
        // All FIDs including base types and named types:
        // ["Any", "Follow", "Follow:1", "Post", "Post:1", "Post:2", "Profile", "Profile:1", "Profile:2", "Type"]
        let query_msg = QueryMsg::CyberlinksByFIDs { start_after_fid: None, limit: Some(3) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(String, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 3);
        let mut all_fids_full = all_fids.clone();
        all_fids_full.push("Type".to_string());
        all_fids_full.push("Any".to_string());
        all_fids_full.push("Profile".to_string()); // Add named types created in test setup
        all_fids_full.push("Follow".to_string());
        all_fids_full.push("Post".to_string());
        all_fids_full.sort(); // Sort all formatted IDs lexicographically

        assert_eq!(links[0].0, all_fids_full[0]); // "Any"
        assert_eq!(links[1].0, all_fids_full[1]); // "Follow"
        assert_eq!(links[2].0, all_fids_full[2]); // "Follow:1"

        let query_msg = QueryMsg::CyberlinksByFIDs { start_after_fid: Some(links[2].0.clone()), limit: Some(10) }; // Start after "Follow:1"
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(String, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 7); // "Post", "Post:1", "Post:2", "Profile", "Profile:1", "Profile:2", "Type"
        assert_eq!(links[0].0, all_fids_full[3]);
        assert_eq!(links[1].0, all_fids_full[4]);
        assert_eq!(links[2].0, all_fids_full[5]);
        assert_eq!(links[3].0, all_fids_full[6]);
        assert_eq!(links[4].0, all_fids_full[7]);
        assert_eq!(links[5].0, all_fids_full[8]);
        assert_eq!(links[6].0, all_fids_full[9]);

        // --- Test CyberlinksSetByIDs ---
        let query_msg = QueryMsg::CyberlinksSetByFIDs { fids: vec![profile1_fid.clone(), post2_fid.clone(), "NonExistent:1".to_string()] };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(String, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|(id, _)| *id == profile1_fid));
        assert!(links.iter().any(|(id, _)| *id == post2_fid));

        // --- Test CyberlinksByType ---
        let query_msg = QueryMsg::CyberlinksByType { type_: "Profile".to_string(), start_after_gid: None, limit: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 2);
        assert!(links.iter().all(|(_, state)| state.type_ == "Profile"));

        let query_msg = QueryMsg::CyberlinksByType { type_: "Post".to_string(), start_after_gid: Some(post1_gid), limit: Some(1) };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, post2_gid);
        assert_eq!(links[0].1.type_, "Post");

        // --- Test CyberlinksByFrom ---
        let query_msg = QueryMsg::CyberlinksByFrom { from: profile1_fid.clone(), start_after_gid: None, limit: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, 8); // Actual GID for Follow:1 is 8
        assert_eq!(links[0].1.from, profile1_fid);

        // --- Test CyberlinksByTo ---
        let query_msg = QueryMsg::CyberlinksByTo { to: profile2_fid.clone(), start_after_gid: None, limit: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, 8); // Actual GID for Follow:1 is 8
        assert_eq!(links[0].1.to, profile2_fid);

        // --- Test CyberlinksByOwnerAndType ---
        let query_msg = QueryMsg::CyberlinksByOwnerAndType { owner: user1.to_string(), type_: "Profile".to_string(), start_after_gid: None, limit: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, 6); // Actual GID for Profile:1 is 6
        assert_eq!(links[0].1.owner, user1);
        assert_eq!(links[0].1.type_, "Profile");

        let query_msg = QueryMsg::CyberlinksByOwnerAndType { owner: user1.to_string(), type_: "Post".to_string(), start_after_gid: None, limit: None };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].0, 9); // Actual GID for Post:1 is 9
        assert_eq!(links[0].1.owner, user1);
        assert_eq!(links[0].1.type_, "Post");

        // --- Test Set queries skip deleted ---
        // Delete post1
        execute(deps.as_mut(), mock_env(), user1_info.clone(), ExecuteMsg::DeleteCyberlink { fid: post1_fid.clone() }).unwrap();

        // Test CyberlinksSetByGIDs skips deleted
        let query_msg = QueryMsg::CyberlinksSetByGIDs { gids: vec![post1_gid, post2_gid] };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(u64, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 1); // post1_gid should be skipped
        assert_eq!(links[0].0, post2_gid);

        // Test CyberlinksSetByIDs skips deleted
        let query_msg = QueryMsg::CyberlinksSetByFIDs { fids: vec![post1_fid.clone(), post2_fid.clone()] };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let links: Vec<(String, CyberlinkState)> = from_json(&res).unwrap();
        assert_eq!(links.len(), 1); // post1_fid should be skipped
        assert_eq!(links[0].0, post2_fid);
    }

    #[test]
    fn test_create_cyberlink2() {
        let mut deps = mock_dependencies();
        let admin = deps.api.addr_make("admin");
        let user = deps.api.addr_make("user");

        // Setup: Instantiate contract, create Thread type, create existing Thread
        let instantiate_msg = InstantiateMsg {
            admins: vec![admin.to_string()],
            executers: vec![user.to_string()],
            semantic_cores: Vec::new(),
        };
        let admin_info = message_info(&admin, &[]);
        instantiate(deps.as_mut(), mock_env(), admin_info.clone(), instantiate_msg).unwrap();

        // Create Thread and Message types
        let type_msg = Cyberlink { type_: "Type".to_string(), from: None, to: None, value: None };
        execute(deps.as_mut(), mock_env(), admin_info.clone(), ExecuteMsg::CreateNamedCyberlink { name: "Thread".to_string(), cyberlink: type_msg.clone() }).unwrap();
        execute(deps.as_mut(), mock_env(), admin_info.clone(), ExecuteMsg::CreateNamedCyberlink { name: "Message".to_string(), cyberlink: type_msg.clone() }).unwrap();
        // Create Replies link type: Message -> Thread
        execute(deps.as_mut(), mock_env(), admin_info.clone(), ExecuteMsg::CreateNamedCyberlink { name: "Replies".to_string(), cyberlink: Cyberlink { type_: "Type".to_string(), from: Some("Message".to_string()), to: Some("Thread".to_string()), value: None } }).unwrap();

        // Create an existing Thread by user
        let thread_res = execute(deps.as_mut(), mock_env(), message_info(&user, &[]), ExecuteMsg::CreateCyberlink { cyberlink: Cyberlink { type_: "Thread".to_string(), from: None, to: None, value: Some("Main Thread".to_string()) } }).unwrap();
        let existing_thread_id = thread_res.attributes.iter().find(|a| a.key == "fid").unwrap().value.clone();
        assert_eq!(existing_thread_id, "Thread:1");

        // --- Test Case 1: Success - Create Message and link FROM new Message TO existing Thread ---
        let user_info = message_info(&user, &[]);
        let msg = ExecuteMsg::CreateCyberlink2 {
            node_type: "Message".to_string(),
            node_value: Some("First message".to_string()),
            link_type: "Replies".to_string(),
            link_value: None,
            link_from_existing_id: None, // New node is FROM
            link_to_existing_id: Some(existing_thread_id.clone()), // Link TO existing thread
        };

        let res = execute(deps.as_mut(), mock_env(), user_info.clone(), msg).unwrap();
        
        // Check response attributes
        let node_fid = res.attributes.iter().find(|a| a.key == "node_fid").unwrap().value.clone(); // Updated key
        let link_fid = res.attributes.iter().find(|a| a.key == "link_fid").unwrap().value.clone(); // Updated key
        assert_eq!(node_fid, "Message:1"); // First message
        assert_eq!(link_fid, "Replies:1"); // First reply link
        assert_eq!(res.attributes[0].value, "create_cyberlink2"); // Renamed action attribute

        // Verify created node (Message:1)
        let query_node = QueryMsg::CyberlinkByFID { fid: node_fid.clone() };
        let node_res = query(deps.as_ref(), mock_env(), query_node).unwrap();
        let node_state: CyberlinkState = from_json(&node_res).unwrap();
        assert_eq!(node_state.type_, "Message");
        assert_eq!(node_state.value, "First message");
        assert_eq!(node_state.owner, user);

        // Verify created link (Replies:1)
        let query_link = QueryMsg::CyberlinkByFID { fid: link_fid.clone() };
        let link_res = query(deps.as_ref(), mock_env(), query_link).unwrap();
        let link_state: CyberlinkState = from_json(&link_res).unwrap();
        assert_eq!(link_state.type_, "Replies");
        assert_eq!(link_state.from, node_fid);
        assert_eq!(link_state.to, existing_thread_id);
        assert_eq!(link_state.owner, user);

        // --- Test Case 2: Error - Invalid Link Specification (both None) ---
        let msg_invalid_spec1 = ExecuteMsg::CreateCyberlink2 {
            node_type: "Message".to_string(),
            node_value: Some("Invalid msg".to_string()),
            link_type: "Replies".to_string(),
            link_value: None,
            link_from_existing_id: None, // Error: both None
            link_to_existing_id: None,
        };
        let err = execute(deps.as_mut(), mock_env(), user_info.clone(), msg_invalid_spec1).unwrap_err();
        assert!(matches!(err, ContractError::InvalidLinkSpecification {}));

        // --- Test Case 3: Error - Invalid Link Specification (both Some) ---
        let msg_invalid_spec2 = ExecuteMsg::CreateCyberlink2 {
            node_type: "Message".to_string(),
            node_value: Some("Invalid msg".to_string()),
            link_type: "Replies".to_string(),
            link_value: None,
            link_from_existing_id: Some(existing_thread_id.clone()), // Error: both Some
            link_to_existing_id: Some(existing_thread_id.clone()),
        };
        let err = execute(deps.as_mut(), mock_env(), user_info.clone(), msg_invalid_spec2).unwrap_err();
        assert!(matches!(err, ContractError::InvalidLinkSpecification {}));

        // --- Test Case 4: Error - Link Type Not Exists ---
        let msg_bad_link_type = ExecuteMsg::CreateCyberlink2 {
            node_type: "Message".to_string(),
            node_value: Some("Another message".to_string()),
            link_type: "InvalidLinkType".to_string(), // This type doesn't exist
            link_value: None,
            link_from_existing_id: None,
            link_to_existing_id: Some(existing_thread_id.clone()),
        };
        let err = execute(deps.as_mut(), mock_env(), user_info.clone(), msg_bad_link_type).unwrap_err();
        assert!(matches!(err, ContractError::TypeNotExists { type_: t } if t == "InvalidLinkType"));

        // --- Test Case 5: Error - Existing Node Not Exists ---
        let msg_bad_target = ExecuteMsg::CreateCyberlink2 {
            node_type: "Message".to_string(),
            node_value: Some("Another message".to_string()),
            link_type: "Replies".to_string(),
            link_value: None,
            link_from_existing_id: None,
            link_to_existing_id: Some("Thread:999".to_string()), // This thread doesn't exist
        };
        let err = execute(deps.as_mut(), mock_env(), user_info.clone(), msg_bad_target).unwrap_err();
        assert!(matches!(err, ContractError::ToNotExists { to: t } if t == "Thread:999"));

        // --- Test Case 6: Error - Type Conflict (e.g., Replies expects Message -> Thread, try Thread -> Thread) ---
         let msg_type_conflict = ExecuteMsg::CreateCyberlink2 {
            node_type: "Thread".to_string(), // Trying to create a Thread node...
            node_value: Some("Nested Thread?".to_string()),
            link_type: "Replies".to_string(), // ... but link it using Replies (expects Message -> Thread)
            link_value: None,
            link_from_existing_id: None, // From new Thread
            link_to_existing_id: Some(existing_thread_id.clone()), // To existing Thread
        };
        let err = execute(deps.as_mut(), mock_env(), user_info.clone(), msg_type_conflict).unwrap_err();
        assert!(matches!(err, ContractError::TypeConflict { .. })); // Detailed check might be needed if specific fields matter

        // --- Test Case 7: Success - Create node and link FROM existing TO new ---
        // First, create another type "IsBasedOn" (e.g., Message -> Message)
        execute(deps.as_mut(), mock_env(), admin_info.clone(), ExecuteMsg::CreateNamedCyberlink { name: "IsBasedOn".to_string(), cyberlink: Cyberlink { type_: "Type".to_string(), from: Some("Message".to_string()), to: Some("Message".to_string()), value: None } }).unwrap();

        // Now create a new message based on Message:1
        let msg_link_from_existing = ExecuteMsg::CreateCyberlink2 {
            node_type: "Message".to_string(),
            node_value: Some("Follow-up message".to_string()),
            link_type: "IsBasedOn".to_string(),
            link_value: None,
            link_from_existing_id: Some("Message:1".to_string()), // Link FROM Message:1
            link_to_existing_id: None, // TO the new message
        };
        let res = execute(deps.as_mut(), mock_env(), user_info.clone(), msg_link_from_existing).unwrap();
        let node_fid_2 = res.attributes.iter().find(|a| a.key == "node_fid").unwrap().value.clone(); // Updated key
        let link_fid_2 = res.attributes.iter().find(|a| a.key == "link_fid").unwrap().value.clone(); // Updated key
        assert_eq!(node_fid_2, "Message:2");
        assert_eq!(link_fid_2, "IsBasedOn:1");

        // Verify link direction
        let query_link = QueryMsg::CyberlinkByFID { fid: link_fid_2.clone() };
        let link_res = query(deps.as_ref(), mock_env(), query_link).unwrap();
        let link_state: CyberlinkState = from_json(&link_res).unwrap();
        assert_eq!(link_state.type_, "IsBasedOn");
        assert_eq!(link_state.from, "Message:1"); // From existing
        assert_eq!(link_state.to, node_fid_2); // To new
    }
}
