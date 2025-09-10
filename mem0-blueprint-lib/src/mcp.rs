use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<McpError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpMemoryTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

pub struct McpServer {
    pub tools: Vec<McpMemoryTool>,
}

impl McpServer {
    pub fn new() -> Self {
        let tools = vec![
            McpMemoryTool {
                name: "add_memory".to_string(),
                description: "Add a new memory to the memory store".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content": {
                            "type": "string",
                            "description": "The content to store in memory"
                        },
                        "user_id": {
                            "type": "string",
                            "description": "Optional user ID to associate with the memory"
                        },
                        "agent_id": {
                            "type": "string",
                            "description": "Optional agent ID to associate with the memory"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Optional session ID to associate with the memory"
                        },
                        "metadata": {
                            "type": "object",
                            "description": "Optional metadata key-value pairs"
                        }
                    },
                    "required": ["content"]
                }),
            },
            McpMemoryTool {
                name: "search_memory".to_string(),
                description: "Search for memories based on content and filters".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The search query to match against memory content"
                        },
                        "user_id": {
                            "type": "string",
                            "description": "Optional user ID filter"
                        },
                        "agent_id": {
                            "type": "string",
                            "description": "Optional agent ID filter"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Optional session ID filter"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 10)"
                        }
                    },
                    "required": ["query"]
                }),
            },
            McpMemoryTool {
                name: "get_memory".to_string(),
                description: "Retrieve a specific memory by ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_id": {
                            "type": "string",
                            "description": "The ID of the memory to retrieve"
                        }
                    },
                    "required": ["memory_id"]
                }),
            },
            McpMemoryTool {
                name: "update_memory".to_string(),
                description: "Update an existing memory".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_id": {
                            "type": "string",
                            "description": "The ID of the memory to update"
                        },
                        "content": {
                            "type": "string",
                            "description": "The new content for the memory"
                        },
                        "metadata": {
                            "type": "object",
                            "description": "Optional metadata to merge with existing metadata"
                        }
                    },
                    "required": ["memory_id", "content"]
                }),
            },
            McpMemoryTool {
                name: "delete_memory".to_string(),
                description: "Delete a memory by ID".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "memory_id": {
                            "type": "string",
                            "description": "The ID of the memory to delete"
                        }
                    },
                    "required": ["memory_id"]
                }),
            },
            McpMemoryTool {
                name: "get_all_memories".to_string(),
                description: "Retrieve all memories with optional filtering".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "user_id": {
                            "type": "string",
                            "description": "Optional user ID filter"
                        },
                        "agent_id": {
                            "type": "string",
                            "description": "Optional agent ID filter"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Optional session ID filter"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 100)"
                        }
                    }
                }),
            },
        ];

        Self { tools }
    }

    pub fn handle_request(&self, request: McpRequest) -> McpResponse {
        match request.method.as_str() {
            "tools/list" => self.list_tools(request.id),
            "tools/call" => self.call_tool(request.id, request.params),
            _ => McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        }
    }

    fn list_tools(&self, id: Option<serde_json::Value>) -> McpResponse {
        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({
                "tools": self.tools
            })),
            error: None,
        }
    }

    fn call_tool(&self, id: Option<serde_json::Value>, params: Option<serde_json::Value>) -> McpResponse {
        let params = match params {
            Some(p) => p,
            None => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: "Invalid params".to_string(),
                        data: None,
                    }),
                };
            }
        };

        let tool_name = match params.get("name").and_then(|n| n.as_str()) {
            Some(name) => name,
            None => {
                return McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(McpError {
                        code: -32602,
                        message: "Tool name required".to_string(),
                        data: None,
                    }),
                };
            }
        };

        McpResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::json!({
                "content": [{
                    "type": "text",
                    "text": format!("Tool {} called successfully. This would be handled by the blueprint job system.", tool_name)
                }]
            })),
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_list_tools() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = server.handle_request(request);
        assert!(response.error.is_none());
        assert!(response.result.is_some());
        
        let result = response.result.unwrap();
        let tools = result.get("tools").unwrap().as_array().unwrap();
        assert_eq!(tools.len(), 6);
    }

    #[test]
    fn test_mcp_server_unknown_method() {
        let server = McpServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "unknown/method".to_string(),
            params: None,
        };

        let response = server.handle_request(request);
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);
    }
}
