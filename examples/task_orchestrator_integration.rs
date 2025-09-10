use mem0_blueprint_lib::benchmarks::{BenchmarkConfig, MemoryBenchmark, SearchComplexity};
use serde_json;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Task Orchestrator Integration Example");
    println!("====================================");

    let configs = vec![
        ("Light Load", BenchmarkConfig {
            num_operations: 100,
            concurrent_operations: 5,
            memory_content_size: 50,
            search_query_complexity: SearchComplexity::Simple,
            delay_between_operations: Duration::from_millis(10),
        }),
        ("Medium Load", BenchmarkConfig {
            num_operations: 500,
            concurrent_operations: 10,
            memory_content_size: 100,
            search_query_complexity: SearchComplexity::Medium,
            delay_between_operations: Duration::from_millis(5),
        }),
        ("Heavy Load", BenchmarkConfig {
            num_operations: 1000,
            concurrent_operations: 20,
            memory_content_size: 200,
            search_query_complexity: SearchComplexity::Complex,
            delay_between_operations: Duration::from_millis(1),
        }),
    ];

    let mut all_results = Vec::new();

    for (name, config) in configs {
        println!("\nRunning {} benchmark...", name);
        let benchmark = MemoryBenchmark::new(config);
        let suite = benchmark.run_full_benchmark().await;

        println!("Completed in {:.2?}", suite.total_duration);
        
        let avg_ops_per_sec: f64 = suite.results.iter()
            .map(|r| r.operations_per_second)
            .sum::<f64>() / suite.results.len() as f64;
        
        println!("Average operations/sec: {:.2}", avg_ops_per_sec);
        
        all_results.push((name, suite));
    }

    println!("\n=== Task Orchestrator Summary ===");
    println!("Benchmark,Total_Duration_ms,Avg_Ops_Per_Sec,Success_Rate");
    
    for (name, suite) in &all_results {
        let total_duration_ms = suite.total_duration.as_millis();
        let avg_ops_per_sec: f64 = suite.results.iter()
            .map(|r| r.operations_per_second)
            .sum::<f64>() / suite.results.len() as f64;
        let avg_success_rate: f64 = suite.results.iter()
            .map(|r| r.success_rate)
            .sum::<f64>() / suite.results.len() as f64;
        
        println!("{},{},{:.2},{:.4}", name, total_duration_ms, avg_ops_per_sec, avg_success_rate);
    }

    let json_output = serde_json::to_string_pretty(&all_results.iter().map(|(_, suite)| suite).collect::<Vec<_>>())?;
    std::fs::write("benchmark_results.json", json_output)?;
    println!("\nDetailed results saved to benchmark_results.json");

    println!("\n=== Integration Points for Task Orchestrator ===");
    println!("1. Memory operations can be called via blueprint jobs (IDs 0-5)");
    println!("2. Benchmark results provide performance baselines");
    println!("3. MCP protocol compatibility enables AI agent integration");
    println!("4. Concurrent operation support for high-throughput scenarios");
    println!("5. JSON/CSV output formats for automated analysis");

    Ok(())
}
