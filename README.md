# Memory MCP Server Blueprint

## Overview

This Tangle Blueprint implements a Memory Context Protocol (MCP) server that provides intelligent memory management capabilities. It's designed to work with AI agents and applications that need persistent, searchable memory storage.

The blueprint implements the mem0 memory architecture patterns and provides a complete MCP-compatible server for memory operations. It can be used as a self-hosted alternative to managed memory services, giving you full control over your AI agent's memory data.

For more details about Tangle Blueprints, refer to the [project documentation](https://docs.tangle.tools/developers/blueprints/introduction).

## Features

- **Memory Operations**: Add, search, get, update, delete, and list memories
- **Multi-level Memory**: Support for user, agent, and session-scoped memories
- **Metadata Support**: Rich metadata storage and filtering capabilities
- **MCP Protocol**: Full Model Context Protocol compatibility
- **High Performance**: Optimized for concurrent memory operations
- **Benchmarking**: Built-in performance benchmarking tools
- **Task Orchestrator Compatible**: Ready for integration with task orchestration systems

## 📋 Prerequisites

Before you can run this project, you will need to have the following software installed on your machine:

- [Rust](https://www.rust-lang.org/tools/install)
- [Forge](https://getfoundry.sh)

You will also need to install [cargo-tangle](https://crates.io/crates/cargo-tangle), our CLI tool for creating and
deploying Tangle Blueprints:

To install the Tangle CLI, run the following command:

> Supported on Linux, MacOS, and Windows (WSL2)

```bash
cargo install cargo-tangle --git https://github.com/tangle-network/blueprint
```

## Getting Started

### Running the Memory MCP Server

Build and run the memory server:

```sh
cargo build --release
cargo run --bin mem0-blueprint-bin run
```

### Running Benchmarks

The blueprint includes comprehensive benchmarking tools compatible with task orchestrators:

```sh
# Run basic benchmark
cargo run --bin mem0-blueprint-bin benchmark

# Run with custom parameters
cargo run --bin mem0-blueprint-bin benchmark \
  --operations 5000 \
  --concurrent 20 \
  --content-size 200 \
  --complexity complex \
  --delay-ms 0

# Output results as JSON
cargo run --bin mem0-blueprint-bin benchmark --json-output

# Output results as CSV for analysis
cargo run --bin mem0-blueprint-bin benchmark --csv-output
```

### Memory Operations

The blueprint supports the following memory operations:

1. **Add Memory** (Job ID: 0): Store new memories with content, user/agent/session context, and metadata
2. **Search Memory** (Job ID: 1): Search memories by content with filtering by user/agent/session
3. **Get Memory** (Job ID: 2): Retrieve a specific memory by ID
4. **Update Memory** (Job ID: 3): Update memory content and metadata
5. **Delete Memory** (Job ID: 4): Remove a memory by ID
6. **Get All Memories** (Job ID: 5): List all memories with optional filtering

### MCP Integration

The blueprint implements the Model Context Protocol (MCP) specification, making it compatible with MCP clients. The server provides tools for:

- `add_memory`: Add new memories to the store
- `search_memory`: Search existing memories
- `get_memory`: Retrieve specific memories
- `update_memory`: Update memory content
- `delete_memory`: Remove memories
- `get_all_memories`: List all memories

### Deployment

Deploy the blueprint to the Tangle network:

```sh
cargo tangle blueprint deploy
```

## 📜 License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 📬 Feedback and Contributions

We welcome feedback and contributions to improve this blueprint.
Please open an issue or submit a pull request on our GitHub repository.
Please let us know if you fork this blueprint and extend it too!

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
