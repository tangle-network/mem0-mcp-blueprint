use blueprint_sdk::Job;
use blueprint_sdk::tangle::layers::TangleLayer;
use blueprint_sdk::testing::tempfile;
use blueprint_sdk::testing::utils::setup_log;
use blueprint_sdk::testing::utils::tangle::TangleTestHarness;
use blueprint_sdk::tangle::serde::to_field;
use mem0_blueprint_lib::{
    MemoryContext, AddMemoryRequest, SearchMemoryRequest, GetMemoryRequest,
    add_memory, search_memory, get_memory
};
use std::collections::HashMap;

const N: usize = 1;

#[tokio::test]
async fn test_add_memory() -> color_eyre::Result<()> {
    setup_log();

    let temp_dir = tempfile::TempDir::new()?;
    let context = MemoryContext::new();
    let harness = TangleTestHarness::setup(temp_dir).await?;

    let (mut test_env, service_id, _) = harness.setup_services::<N>(false).await?;

    test_env.initialize().await?;
    test_env.add_job(add_memory.layer(TangleLayer)).await;

    test_env.start(context).await?;

    let add_request = AddMemoryRequest {
        content: "User prefers dark mode".to_string(),
        user_id: Some("user123".to_string()),
        agent_id: None,
        session_id: None,
        metadata: Some(HashMap::from([("category".to_string(), "preference".to_string())])),
    };

    let job_inputs = vec![to_field(&add_request).unwrap()];
    let job = harness.submit_job(service_id, 0, job_inputs).await?;

    let results = harness.wait_for_job_execution(service_id, job).await?;

    assert_eq!(results.service_id, service_id);
    Ok(())
}

#[tokio::test]
async fn test_search_memory() -> color_eyre::Result<()> {
    setup_log();

    let temp_dir = tempfile::TempDir::new()?;
    let context = MemoryContext::new();
    let harness = TangleTestHarness::setup(temp_dir).await?;

    let (mut test_env, service_id, _) = harness.setup_services::<N>(false).await?;

    test_env.initialize().await?;
    test_env.add_job(add_memory.layer(TangleLayer)).await;
    test_env.add_job(search_memory.layer(TangleLayer)).await;

    test_env.start(context).await?;

    let add_request = AddMemoryRequest {
        content: "User prefers dark mode".to_string(),
        user_id: Some("user123".to_string()),
        agent_id: None,
        session_id: None,
        metadata: Some(HashMap::from([("category".to_string(), "preference".to_string())])),
    };

    let add_inputs = vec![to_field(&add_request).unwrap()];
    let add_job = harness.submit_job(service_id, 0, add_inputs).await?;
    harness.wait_for_job_execution(service_id, add_job).await?;

    let search_request = SearchMemoryRequest {
        query: "dark mode".to_string(),
        user_id: Some("user123".to_string()),
        agent_id: None,
        session_id: None,
        limit: Some(10),
    };

    let search_inputs = vec![to_field(&search_request).unwrap()];
    let search_job = harness.submit_job(service_id, 1, search_inputs).await?;

    let results = harness.wait_for_job_execution(service_id, search_job).await?;

    assert_eq!(results.service_id, service_id);
    Ok(())
}

#[tokio::test]
async fn test_get_memory() -> color_eyre::Result<()> {
    setup_log();

    let temp_dir = tempfile::TempDir::new()?;
    let context = MemoryContext::new();
    let harness = TangleTestHarness::setup(temp_dir).await?;

    let (mut test_env, service_id, _) = harness.setup_services::<N>(false).await?;

    test_env.initialize().await?;
    test_env.add_job(add_memory.layer(TangleLayer)).await;
    test_env.add_job(get_memory.layer(TangleLayer)).await;

    test_env.start(context).await?;

    let add_request = AddMemoryRequest {
        content: "Important meeting notes".to_string(),
        user_id: Some("user456".to_string()),
        agent_id: None,
        session_id: None,
        metadata: None,
    };

    let add_inputs = vec![to_field(&add_request).unwrap()];
    let add_job = harness.submit_job(service_id, 0, add_inputs).await?;
    let add_results = harness.wait_for_job_execution(service_id, add_job).await?;

    let get_request = GetMemoryRequest {
        memory_id: "test_memory_id".to_string(),
    };

    let get_inputs = vec![to_field(&get_request).unwrap()];
    let get_job = harness.submit_job(service_id, 2, get_inputs).await?;

    let results = harness.wait_for_job_execution(service_id, get_job).await?;

    assert_eq!(results.service_id, service_id);
    Ok(())
}
