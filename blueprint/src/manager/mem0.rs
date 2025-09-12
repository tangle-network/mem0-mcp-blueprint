use std::collections::BTreeMap;
use std::path::PathBuf;
use futures::TryFutureExt;
use tokio::fs;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::SupportedTransportAdapter;
use crate::error::Error;
use crate::manager::McpRunner;
use crate::Mem0Config;

/// Mem0 memory server runner
/// 
/// This runner manages the mem0 self-hosted memory server using Docker Compose.
/// It sets up PostgreSQL with pgvector, Neo4j for graph memory, and the mem0 API server.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Mem0Runner;

impl Mem0Runner {
    /// Create Docker Compose configuration for mem0 services
    async fn create_docker_compose_config(
        &self,
        service_id: u64,
        mem0_config: &Mem0Config,
        allocated_port: u16,
        env_vars: &BTreeMap<String, String>,
    ) -> Result<String, Error> {
        let postgres_config = &mem0_config.postgres_config;
        let neo4j_config = &mem0_config.neo4j_config;
        let api_config = &mem0_config.api_config;

        let compose_config = format!(
            r#"version: '3.8'

services:
  postgres:
    image: pgvector/pgvector:pg16
    container_name: mem0-postgres-{service_id}
    environment:
      POSTGRES_DB: {postgres_db}
      POSTGRES_USER: {postgres_user}
      POSTGRES_PASSWORD: {postgres_password}
    ports:
      - "127.0.0.1:{postgres_port}:{postgres_port}"
    volumes:
      - mem0_postgres_data_{service_id}:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U {postgres_user} -d {postgres_db}"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - mem0_network_{service_id}

  neo4j:
    image: neo4j:5.15-community
    container_name: mem0-neo4j-{service_id}
    environment:
      NEO4J_AUTH: {neo4j_user}/{neo4j_password}
      NEO4J_PLUGINS: '["apoc"]'
      NEO4J_apoc_export_file_enabled: true
      NEO4J_apoc_import_file_enabled: true
      NEO4J_apoc_import_file_use__neo4j__config: true
    ports:
      - "127.0.0.1:{neo4j_http_port}:7474"
      - "127.0.0.1:{neo4j_bolt_port}:7687"
    volumes:
      - mem0_neo4j_data_{service_id}:/data
      - mem0_neo4j_logs_{service_id}:/logs
    healthcheck:
      test: ["CMD-SHELL", "cypher-shell -u {neo4j_user} -p {neo4j_password} 'RETURN 1'"]
      interval: 10s
      timeout: 5s
      retries: 5
    networks:
      - mem0_network_{service_id}

  mem0-api:
    image: mem0ai/mem0:latest
    container_name: mem0-api-{service_id}
    environment:
      OPENAI_API_KEY: {openai_api_key}
      POSTGRES_URL: postgresql://{postgres_user}:{postgres_password}@postgres:{postgres_port}/{postgres_db}
      NEO4J_URL: bolt://neo4j:{neo4j_bolt_port}
      NEO4J_USER: {neo4j_user}
      NEO4J_PASSWORD: {neo4j_password}
      PORT: {allocated_port}
      HOST: {api_host}
{env_vars_section}
    ports:
      - "127.0.0.1:{allocated_port}:{allocated_port}"
    depends_on:
      postgres:
        condition: service_healthy
      neo4j:
        condition: service_healthy
    networks:
      - mem0_network_{service_id}
    command: ["python", "-m", "mem0.server"]

{mcp_server_section}

volumes:
  mem0_postgres_data_{service_id}:
  mem0_neo4j_data_{service_id}:
  mem0_neo4j_logs_{service_id}:

networks:
  mem0_network_{service_id}:
    driver: bridge
"#,
            service_id = service_id,
            postgres_db = postgres_config.database,
            postgres_user = postgres_config.user,
            postgres_password = postgres_config.password,
            postgres_port = postgres_config.port,
            neo4j_user = neo4j_config.user,
            neo4j_password = neo4j_config.password,
            neo4j_http_port = neo4j_config.port + 1000, // 7474 -> 8474 to avoid conflicts
            neo4j_bolt_port = neo4j_config.port,
            openai_api_key = mem0_config.openai_api_key,
            allocated_port = allocated_port,
            api_host = api_config.host,
            env_vars_section = Self::format_env_vars(env_vars),
            mcp_server_section = if mem0_config.enable_mcp_server {
                format!(
                    r#"
  openmemory-mcp:
    image: mem0ai/openmemory:latest
    container_name: mem0-mcp-{service_id}
    environment:
      OPENAI_API_KEY: {openai_api_key}
      MEM0_API_URL: http://mem0-api:{allocated_port}
      PORT: {mcp_port}
    ports:
      - "127.0.0.1:{mcp_port}:{mcp_port}"
    depends_on:
      - mem0-api
    networks:
      - mem0_network_{service_id}
"#,
                    service_id = service_id,
                    openai_api_key = mem0_config.openai_api_key,
                    allocated_port = allocated_port,
                    mcp_port = allocated_port + 1000, // MCP server on different port
                )
            } else {
                String::new()
            }
        );

        Ok(compose_config)
    }

    /// Format environment variables for Docker Compose
    fn format_env_vars(env_vars: &BTreeMap<String, String>) -> String {
        env_vars
            .iter()
            .filter(|(k, _)| k.as_str() != "PORT") // PORT is handled separately
            .map(|(k, v)| format!("      {k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Create a temporary directory for this service's Docker Compose files
    async fn create_service_directory(&self, service_id: u64) -> Result<PathBuf, Error> {
        let service_dir = std::env::temp_dir().join(format!("mem0-service-{service_id}"));
        fs::create_dir_all(&service_dir).await.map_err(Error::Io)?;
        Ok(service_dir)
    }

    /// Write Docker Compose configuration to file
    async fn write_compose_file(
        &self,
        service_dir: &PathBuf,
        compose_config: &str,
    ) -> Result<PathBuf, Error> {
        let compose_file = service_dir.join("docker-compose.yml");
        fs::write(&compose_file, compose_config)
            .await
            .map_err(Error::Io)?;
        Ok(compose_file)
    }

    /// Start Docker Compose services
    async fn start_compose_services(
        &self,
        service_dir: &PathBuf,
    ) -> Result<tokio::process::Child, Error> {
        blueprint_sdk::debug!(?service_dir, "Starting Docker Compose services");
        
        let mut cmd = Command::new("docker-compose");
        cmd.current_dir(service_dir)
            .arg("up")
            .arg("-d")
            .arg("--remove-orphans")
            .kill_on_drop(true)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let child = cmd.spawn().map_err(Error::Io)?;
        blueprint_sdk::debug!("Docker Compose services started");
        Ok(child)
    }

    /// Stop Docker Compose services
    async fn stop_compose_services(&self, service_dir: &PathBuf) -> Result<(), Error> {
        blueprint_sdk::debug!(?service_dir, "Stopping Docker Compose services");
        
        let status = Command::new("docker-compose")
            .current_dir(service_dir)
            .arg("down")
            .arg("-v") // Remove volumes
            .arg("--remove-orphans")
            .status()
            .await
            .map_err(Error::Io)?;

        if status.success() {
            blueprint_sdk::debug!("Docker Compose services stopped successfully");
            // Clean up the service directory
            let _ = fs::remove_dir_all(service_dir).await;
            Ok(())
        } else {
            Err(Error::Io(std::io::Error::other(
                "Failed to stop Docker Compose services",
            )))
        }
    }

    /// Wait for mem0 API to be ready
    async fn wait_for_api_ready(&self, port: u16) -> Result<(), Error> {
        use tokio::time::{sleep, Duration};
        
        let url = format!("http://127.0.0.1:{port}/health");
        let client = reqwest::Client::new();
        
        for attempt in 1..=30 {
            blueprint_sdk::debug!(attempt, %url, "Checking mem0 API health");
            
            match client.get(&url).send().await {
                Ok(response) if response.status().is_success() => {
                    blueprint_sdk::debug!("Mem0 API is ready");
                    return Ok(());
                }
                Ok(response) => {
                    blueprint_sdk::debug!(status = %response.status(), "API not ready yet");
                }
                Err(e) => {
                    blueprint_sdk::debug!(?e, "Failed to connect to API");
                }
            }
            
            sleep(Duration::from_secs(2)).await;
        }
        
        Err(Error::Io(std::io::Error::other(
            "Mem0 API failed to become ready within timeout",
        )))
    }
}

impl McpRunner for Mem0Runner {
    #[tracing::instrument(skip(self, _ctx), fields(%_package, _args, _service_id, _env_vars, runtime = "mem0"))]
    async fn start(
        &self,
        _ctx: &crate::MyContext,
        _service_id: u64,
        _package: String,
        _args: Vec<String>,
        _env_vars: BTreeMap<String, String>,
        _transport_adapter: SupportedTransportAdapter,
    ) -> Result<CancellationToken, Error> {
        // This is a special case for Mem0Runner - we need the mem0_config
        Err(Error::Io(std::io::Error::other(
            "Mem0Runner requires mem0_config parameter. Use start_with_config instead.",
        )))
    }

    async fn check(&self, _ctx: &crate::MyContext) -> Result<bool, Error> {
        // Check if docker-compose is available
        let status = Command::new("docker-compose")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map_err(Error::Io)
            .await?;
        Ok(status.success())
    }

    #[tracing::instrument(skip(self, _ctx), fields(runtime = "mem0"))]
    async fn install(&self, _ctx: &crate::MyContext) -> Result<(), Error> {
        // Docker Compose installation is platform-specific and complex
        // For now, we assume it's already installed or will be installed by the user
        Err(Error::Io(std::io::Error::other(
            "Docker Compose installation not implemented. Please install Docker Compose manually.",
        )))
    }
}

impl Mem0Runner {
    /// Start mem0 services with mem0-specific configuration
    #[tracing::instrument(skip(self, ctx), fields(service_id, runtime = "mem0"))]
    pub async fn start(
        &self,
        ctx: &crate::MyContext,
        service_id: u64,
        _package: String,
        _args: Vec<String>,
        env_vars: BTreeMap<String, String>,
        _transport_adapter: SupportedTransportAdapter,
        mem0_config: Option<Mem0Config>,
    ) -> Result<CancellationToken, Error> {
        let mem0_config = mem0_config.ok_or_else(|| {
            Error::Io(std::io::Error::other(
                "Mem0 configuration is required for Mem0 runtime",
            ))
        })?;

        // Ensure docker-compose is available
        let checked = self.check(ctx).await;
        blueprint_sdk::debug!(?checked, "Checking if docker-compose is available");
        if !matches!(checked, Ok(true)) {
            return Err(Error::Io(std::io::Error::other(
                "docker-compose is not available. Please install Docker Compose.",
            )));
        }

        let allocated_port = env_vars
            .get("PORT")
            .and_then(|p| p.parse::<u16>().ok())
            .ok_or(Error::MissingPortBinding)?;

        // Create service directory
        let service_dir = self.create_service_directory(service_id).await?;
        
        // Create Docker Compose configuration
        let compose_config = self
            .create_docker_compose_config(service_id, &mem0_config, allocated_port, &env_vars)
            .await?;
        
        // Write compose file
        let _compose_file = self.write_compose_file(&service_dir, &compose_config).await?;
        
        // Start services
        let mut compose_process = self.start_compose_services(&service_dir).await?;
        
        // Wait for API to be ready
        self.wait_for_api_ready(allocated_port).await?;

        let ct = CancellationToken::new();
        let cleanup_ct = ct.clone();
        let cleanup_service_dir = service_dir.clone();
        let cleanup_runner = self.clone();

        // Spawn cleanup task
        tokio::spawn(async move {
            cleanup_ct.cancelled().await;
            blueprint_sdk::debug!(?cleanup_service_dir, "Cleaning up mem0 services");
            
            // Kill the compose process
            if let Err(e) = compose_process.kill().await {
                blueprint_sdk::error!(?e, "Failed to kill compose process");
            }
            
            // Stop compose services
            if let Err(e) = cleanup_runner.stop_compose_services(&cleanup_service_dir).await {
                blueprint_sdk::error!(?e, "Failed to stop compose services");
            }
        });

        blueprint_sdk::debug!(%allocated_port, "Mem0 services started successfully");
        Ok(ct)
    }
}
