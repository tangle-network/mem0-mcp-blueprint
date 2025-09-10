use mem0_blueprint_lib::{
    MemoryContext, AddMemoryRequest, SearchMemoryRequest, GetMemoryRequest,
    UpdateMemoryRequest, DeleteMemoryRequest, GetAllMemoriesRequest
};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Memory MCP Server - Example Operations");
    println!("=====================================");

    let context = MemoryContext::new();

    println!("\n1. Adding memories...");
    let memory1 = context.add_memory(AddMemoryRequest {
        content: "User prefers dark mode in the IDE".to_string(),
        user_id: Some("user123".to_string()),
        agent_id: Some("coding_assistant".to_string()),
        session_id: Some("session_001".to_string()),
        metadata: Some(HashMap::from([
            ("category".to_string(), "preference".to_string()),
            ("priority".to_string(), "high".to_string()),
        ])),
    }).await;
    println!("Added memory: {} - {}", memory1.id, memory1.content);

    let memory2 = context.add_memory(AddMemoryRequest {
        content: "User is working on a Rust blockchain project".to_string(),
        user_id: Some("user123".to_string()),
        agent_id: Some("coding_assistant".to_string()),
        session_id: Some("session_001".to_string()),
        metadata: Some(HashMap::from([
            ("category".to_string(), "project_context".to_string()),
            ("language".to_string(), "rust".to_string()),
        ])),
    }).await;
    println!("Added memory: {} - {}", memory2.id, memory2.content);

    let memory3 = context.add_memory(AddMemoryRequest {
        content: "User frequently uses async/await patterns".to_string(),
        user_id: Some("user123".to_string()),
        agent_id: Some("coding_assistant".to_string()),
        session_id: Some("session_002".to_string()),
        metadata: Some(HashMap::from([
            ("category".to_string(), "coding_pattern".to_string()),
            ("language".to_string(), "rust".to_string()),
        ])),
    }).await;
    println!("Added memory: {} - {}", memory3.id, memory3.content);

    println!("\n2. Searching memories...");
    let search_results = context.search_memories(SearchMemoryRequest {
        query: "rust".to_string(),
        user_id: Some("user123".to_string()),
        agent_id: None,
        session_id: None,
        limit: Some(10),
    }).await;
    println!("Found {} memories containing 'rust':", search_results.len());
    for memory in &search_results {
        println!("  - {}: {}", memory.id, memory.content);
    }

    println!("\n3. Getting specific memory...");
    if let Some(retrieved_memory) = context.get_memory(&memory1.id).await {
        println!("Retrieved memory: {}", retrieved_memory.content);
        println!("Metadata: {:?}", retrieved_memory.metadata);
    }

    println!("\n4. Updating memory...");
    let updated_memory = context.update_memory(UpdateMemoryRequest {
        memory_id: memory1.id.clone(),
        content: "User prefers dark mode and high contrast themes".to_string(),
        metadata: Some(HashMap::from([
            ("updated".to_string(), "true".to_string()),
            ("contrast".to_string(), "high".to_string()),
        ])),
    }).await;
    if let Some(updated) = updated_memory {
        println!("Updated memory: {}", updated.content);
    }

    println!("\n5. Getting all memories for user...");
    let all_memories = context.get_all_memories(GetAllMemoriesRequest {
        user_id: Some("user123".to_string()),
        agent_id: None,
        session_id: None,
        limit: Some(100),
    }).await;
    println!("Total memories for user123: {}", all_memories.len());

    println!("\n6. Filtering by session...");
    let session_memories = context.get_all_memories(GetAllMemoriesRequest {
        user_id: Some("user123".to_string()),
        agent_id: None,
        session_id: Some("session_001".to_string()),
        limit: Some(100),
    }).await;
    println!("Memories in session_001: {}", session_memories.len());

    println!("\n7. Deleting a memory...");
    let deleted = context.delete_memory(&memory3.id).await;
    println!("Memory deleted: {}", deleted);

    println!("\n8. Final count...");
    let final_memories = context.get_all_memories(GetAllMemoriesRequest {
        user_id: Some("user123".to_string()),
        agent_id: None,
        session_id: None,
        limit: Some(100),
    }).await;
    println!("Remaining memories: {}", final_memories.len());

    println!("\nExample completed successfully!");
    Ok(())
}
