use std::path::Path;

fn main() {
    println!("cargo::rerun-if-changed=../mem0-blueprint-lib");
    
    // Create a simple blueprint.json for now
    let blueprint_json = r#"{
  "name": "mem0-blueprint",
  "master_manager_revision": "Latest",
  "manager": {
    "Evm": "MemoryBlueprint"
  },
  "jobs": [
    {
      "id": 0,
      "name": "add_memory",
      "description": "Add a new memory to the store"
    },
    {
      "id": 1,
      "name": "search_memory", 
      "description": "Search for memories based on content"
    },
    {
      "id": 2,
      "name": "get_memory",
      "description": "Retrieve a specific memory by ID"
    },
    {
      "id": 3,
      "name": "update_memory",
      "description": "Update an existing memory"
    },
    {
      "id": 4,
      "name": "delete_memory",
      "description": "Delete a memory by ID"
    },
    {
      "id": 5,
      "name": "get_all_memories",
      "description": "Retrieve all memories with optional filtering"
    }
  ]
}"#;

    std::fs::write(
        Path::new(env!("CARGO_WORKSPACE_DIR")).join("blueprint.json"),
        blueprint_json.as_bytes(),
    )
    .unwrap();
}
