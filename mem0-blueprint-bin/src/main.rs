use blueprint_sdk::Job;
use blueprint_sdk::Router;
use blueprint_sdk::contexts::tangle::TangleClientContext;
use blueprint_sdk::crypto::sp_core::SpSr25519;
use blueprint_sdk::crypto::tangle_pair_signer::TanglePairSigner;
use blueprint_sdk::keystore::backends::Backend;
use blueprint_sdk::runner::BlueprintRunner;
use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::runner::tangle::config::TangleConfig;
use blueprint_sdk::tangle::consumer::TangleConsumer;
use blueprint_sdk::tangle::filters::MatchesServiceId;
use blueprint_sdk::tangle::layers::TangleLayer;
use blueprint_sdk::tangle::producer::TangleProducer;
use mem0_blueprint_lib::{
    MemoryContext, ADD_MEMORY_JOB_ID, SEARCH_MEMORY_JOB_ID, GET_MEMORY_JOB_ID,
    UPDATE_MEMORY_JOB_ID, DELETE_MEMORY_JOB_ID, GET_ALL_MEMORIES_JOB_ID,
    add_memory, search_memory, get_memory, update_memory, delete_memory, get_all_memories
};
use tower::filter::FilterLayer;
use tracing::error;
use tracing::level_filters::LevelFilter;
use clap::{Parser, Subcommand};

mod benchmark;

#[derive(Parser)]
#[command(name = "memory-mcp-server")]
#[command(about = "A memory server MCP blueprint for Tangle Network")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Run,
    Benchmark(benchmark::BenchmarkArgs),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_log();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run => run_blueprint().await?,
        Commands::Benchmark(args) => benchmark::run_benchmark(args).await?,
    }

    Ok(())
}

async fn run_blueprint() -> Result<(), blueprint_sdk::Error> {
    let env = BlueprintEnvironment::load()?;
    let sr25519_signer = env.keystore().first_local::<SpSr25519>()?;
    let sr25519_pair = env.keystore().get_secret::<SpSr25519>(&sr25519_signer)?;
    let sr25519_signer = TanglePairSigner::new(sr25519_pair.0);

    let tangle_client = env.tangle_client().await?;
    let tangle_producer =
        TangleProducer::finalized_blocks(tangle_client.rpc_client.clone()).await?;
    let tangle_consumer = TangleConsumer::new(tangle_client.rpc_client.clone(), sr25519_signer);

    let tangle_config = TangleConfig::default();

    let service_id = env.protocol_settings.tangle()?.service_id.unwrap();
    let result = BlueprintRunner::builder(tangle_config, env)
        .router(
            Router::new()
                .route(ADD_MEMORY_JOB_ID, add_memory.layer(TangleLayer))
                .route(SEARCH_MEMORY_JOB_ID, search_memory.layer(TangleLayer))
                .route(GET_MEMORY_JOB_ID, get_memory.layer(TangleLayer))
                .route(UPDATE_MEMORY_JOB_ID, update_memory.layer(TangleLayer))
                .route(DELETE_MEMORY_JOB_ID, delete_memory.layer(TangleLayer))
                .route(GET_ALL_MEMORIES_JOB_ID, get_all_memories.layer(TangleLayer))
                .layer(FilterLayer::new(MatchesServiceId(service_id)))
                .with_context(MemoryContext::new()),
        )
        .producer(tangle_producer)
        .consumer(tangle_consumer)
        .with_shutdown_handler(async { println!("Memory MCP server shutting down...") })
        .run()
        .await;

    if let Err(e) = result {
        error!("Blueprint runner failed: {e:?}");
    }

    Ok(())
}

pub fn setup_log() {
    use tracing_subscriber::util::SubscriberInitExt;

    let _ = tracing_subscriber::fmt::SubscriberBuilder::default()
        .without_time()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::NONE)
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish()
        .try_init();
}
