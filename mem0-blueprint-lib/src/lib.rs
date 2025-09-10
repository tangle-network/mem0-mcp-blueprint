use blueprint_sdk::extract::Context;
use blueprint_sdk::tangle::extract::{TangleArg, TangleResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod mcp;
pub mod benchmarks;

pub const ADD_MEMORY_JOB_ID: u32 = 0;
pub const SEARCH_MEMORY_JOB_ID: u32 = 1;
pub const GET_MEMORY_JOB_ID: u32 = 2;
pub const UPDATE_MEMORY_JOB_ID: u32 = 3;
pub const DELETE_MEMORY_JOB_ID: u32 = 4;
pub const GET_ALL_MEMORIES_JOB_ID: u32 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub content: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddMemoryRequest {
    pub content: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoryRequest {
    pub query: String,
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoryRequest {
    pub memory_id: String,
    pub content: String,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMemoryRequest {
    pub memory_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMemoryRequest {
    pub memory_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAllMemoriesRequest {
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub session_id: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResponse {
    pub success: bool,
    pub memory: Option<Memory>,
    pub memories: Option<Vec<Memory>>,
    pub message: Option<String>,
}

#[derive(Clone)]
pub struct MemoryContext {
    storage: Arc<RwLock<HashMap<String, Memory>>>,
}

impl MemoryContext {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn add_memory(&self, request: AddMemoryRequest) -> Memory {
        let now = chrono::Utc::now().timestamp();
        let memory = Memory {
            id: Uuid::new_v4().to_string(),
            content: request.content,
            user_id: request.user_id,
            agent_id: request.agent_id,
            session_id: request.session_id,
            metadata: request.metadata.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };

        let mut storage = self.storage.write().await;
        storage.insert(memory.id.clone(), memory.clone());
        memory
    }

    async fn search_memories(&self, request: SearchMemoryRequest) -> Vec<Memory> {
        let storage = self.storage.read().await;
        let limit = request.limit.unwrap_or(10) as usize;
        
        storage
            .values()
            .filter(|memory| {
                let content_match = memory.content.to_lowercase().contains(&request.query.to_lowercase());
                let user_match = request.user_id.as_ref().map_or(true, |uid| memory.user_id.as_ref() == Some(uid));
                let agent_match = request.agent_id.as_ref().map_or(true, |aid| memory.agent_id.as_ref() == Some(aid));
                let session_match = request.session_id.as_ref().map_or(true, |sid| memory.session_id.as_ref() == Some(sid));
                
                content_match && user_match && agent_match && session_match
            })
            .take(limit)
            .cloned()
            .collect()
    }

    async fn get_memory(&self, memory_id: &str) -> Option<Memory> {
        let storage = self.storage.read().await;
        storage.get(memory_id).cloned()
    }

    async fn update_memory(&self, request: UpdateMemoryRequest) -> Option<Memory> {
        let mut storage = self.storage.write().await;
        if let Some(memory) = storage.get_mut(&request.memory_id) {
            memory.content = request.content;
            memory.updated_at = chrono::Utc::now().timestamp();
            if let Some(metadata) = request.metadata {
                memory.metadata.extend(metadata);
            }
            Some(memory.clone())
        } else {
            None
        }
    }

    async fn delete_memory(&self, memory_id: &str) -> bool {
        let mut storage = self.storage.write().await;
        storage.remove(memory_id).is_some()
    }

    async fn get_all_memories(&self, request: GetAllMemoriesRequest) -> Vec<Memory> {
        let storage = self.storage.read().await;
        let limit = request.limit.unwrap_or(100) as usize;
        
        storage
            .values()
            .filter(|memory| {
                let user_match = request.user_id.as_ref().map_or(true, |uid| memory.user_id.as_ref() == Some(uid));
                let agent_match = request.agent_id.as_ref().map_or(true, |aid| memory.agent_id.as_ref() == Some(aid));
                let session_match = request.session_id.as_ref().map_or(true, |sid| memory.session_id.as_ref() == Some(sid));
                
                user_match && agent_match && session_match
            })
            .take(limit)
            .cloned()
            .collect()
    }
}

pub async fn add_memory(
    Context(ctx): Context<MemoryContext>,
    TangleArg(request): TangleArg<AddMemoryRequest>,
) -> TangleResult<MemoryResponse> {
    let memory = ctx.add_memory(request).await;
    TangleResult(MemoryResponse {
        success: true,
        memory: Some(memory),
        memories: None,
        message: Some("Memory added successfully".to_string()),
    })
}

pub async fn search_memory(
    Context(ctx): Context<MemoryContext>,
    TangleArg(request): TangleArg<SearchMemoryRequest>,
) -> TangleResult<MemoryResponse> {
    let memories = ctx.search_memories(request).await;
    TangleResult(MemoryResponse {
        success: true,
        memory: None,
        memories: Some(memories),
        message: Some("Search completed successfully".to_string()),
    })
}

pub async fn get_memory(
    Context(ctx): Context<MemoryContext>,
    TangleArg(request): TangleArg<GetMemoryRequest>,
) -> TangleResult<MemoryResponse> {
    match ctx.get_memory(&request.memory_id).await {
        Some(memory) => TangleResult(MemoryResponse {
            success: true,
            memory: Some(memory),
            memories: None,
            message: Some("Memory retrieved successfully".to_string()),
        }),
        None => TangleResult(MemoryResponse {
            success: false,
            memory: None,
            memories: None,
            message: Some("Memory not found".to_string()),
        }),
    }
}

pub async fn update_memory(
    Context(ctx): Context<MemoryContext>,
    TangleArg(request): TangleArg<UpdateMemoryRequest>,
) -> TangleResult<MemoryResponse> {
    match ctx.update_memory(request).await {
        Some(memory) => TangleResult(MemoryResponse {
            success: true,
            memory: Some(memory),
            memories: None,
            message: Some("Memory updated successfully".to_string()),
        }),
        None => TangleResult(MemoryResponse {
            success: false,
            memory: None,
            memories: None,
            message: Some("Memory not found".to_string()),
        }),
    }
}

pub async fn delete_memory(
    Context(ctx): Context<MemoryContext>,
    TangleArg(request): TangleArg<DeleteMemoryRequest>,
) -> TangleResult<MemoryResponse> {
    let success = ctx.delete_memory(&request.memory_id).await;
    TangleResult(MemoryResponse {
        success,
        memory: None,
        memories: None,
        message: Some(if success {
            "Memory deleted successfully".to_string()
        } else {
            "Memory not found".to_string()
        }),
    })
}

pub async fn get_all_memories(
    Context(ctx): Context<MemoryContext>,
    TangleArg(request): TangleArg<GetAllMemoriesRequest>,
) -> TangleResult<MemoryResponse> {
    let memories = ctx.get_all_memories(request).await;
    TangleResult(MemoryResponse {
        success: true,
        memory: None,
        memories: Some(memories),
        message: Some("Memories retrieved successfully".to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_memory_context() {
        let context = MemoryContext::new();
        
        let add_request = AddMemoryRequest {
            content: "Test memory content".to_string(),
            user_id: Some("user123".to_string()),
            agent_id: None,
            session_id: None,
            metadata: Some(HashMap::from([("type".to_string(), "test".to_string())])),
        };

        let memory = context.add_memory(add_request).await;
        assert_eq!(memory.content, "Test memory content");
        assert_eq!(memory.user_id, Some("user123".to_string()));

        let search_request = SearchMemoryRequest {
            query: "test".to_string(),
            user_id: Some("user123".to_string()),
            agent_id: None,
            session_id: None,
            limit: Some(10),
        };

        let memories = context.search_memories(search_request).await;
        assert_eq!(memories.len(), 1);
        assert_eq!(memories[0].content, "Test memory content");

        let retrieved_memory = context.get_memory(&memory.id).await;
        assert!(retrieved_memory.is_some());
        assert_eq!(retrieved_memory.unwrap().id, memory.id);

        let update_request = UpdateMemoryRequest {
            memory_id: memory.id.clone(),
            content: "Updated content".to_string(),
            metadata: None,
        };

        let updated_memory = context.update_memory(update_request).await;
        assert!(updated_memory.is_some());
        assert_eq!(updated_memory.unwrap().content, "Updated content");

        let deleted = context.delete_memory(&memory.id).await;
        assert!(deleted);

        let deleted_memory = context.get_memory(&memory.id).await;
        assert!(deleted_memory.is_none());
    }
}
