use mem0_blueprint_lib::benchmarks::{BenchmarkConfig, MemoryBenchmark, SearchComplexity};
use clap::{Parser, ValueEnum};
use serde_json;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "memory-benchmark")]
#[command(about = "Benchmark tool for memory operations")]
pub struct BenchmarkArgs {
    #[arg(short, long, default_value = "1000")]
    pub operations: usize,

    #[arg(short, long, default_value = "10")]
    pub concurrent: usize,

    #[arg(short = 's', long, default_value = "100")]
    pub content_size: usize,

    #[arg(short = 'c', long, default_value = "medium")]
    pub complexity: ComplexityArg,

    #[arg(short, long, default_value = "1")]
    pub delay_ms: u64,

    #[arg(long)]
    pub json_output: bool,

    #[arg(long)]
    pub csv_output: bool,

    #[arg(short = 'f', long)]
    pub output_file: Option<String>,
}

#[derive(Clone, ValueEnum)]
pub enum ComplexityArg {
    Simple,
    Medium,
    Complex,
}

impl From<ComplexityArg> for SearchComplexity {
    fn from(arg: ComplexityArg) -> Self {
        match arg {
            ComplexityArg::Simple => SearchComplexity::Simple,
            ComplexityArg::Medium => SearchComplexity::Medium,
            ComplexityArg::Complex => SearchComplexity::Complex,
        }
    }
}

pub async fn run_benchmark(args: BenchmarkArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config = BenchmarkConfig {
        num_operations: args.operations,
        concurrent_operations: args.concurrent,
        memory_content_size: args.content_size,
        search_query_complexity: args.complexity.into(),
        delay_between_operations: Duration::from_millis(args.delay_ms),
    };

    println!("Starting memory benchmark with configuration:");
    println!("  Operations: {}", config.num_operations);
    println!("  Concurrent: {}", config.concurrent_operations);
    println!("  Content Size: {} bytes", config.memory_content_size);
    println!("  Search Complexity: {:?}", config.search_query_complexity);
    println!("  Delay: {}ms", args.delay_ms);
    println!();

    let benchmark = MemoryBenchmark::new(config);
    let suite = benchmark.run_full_benchmark().await;

    if args.json_output {
        let json = serde_json::to_string_pretty(&suite)?;
        if let Some(file) = args.output_file {
            std::fs::write(file, json)?;
        } else {
            println!("{json}");
        }
    } else if args.csv_output {
        print_csv_results(&suite);
    } else {
        print_human_readable_results(&suite);
    }

    Ok(())
}

fn print_human_readable_results(suite: &mem0_blueprint_lib::benchmarks::BenchmarkSuite) {
    println!("Benchmark Results");
    println!("=================");
    println!("Total Duration: {:.2?}", suite.total_duration);
    println!();

    for result in &suite.results {
        println!("Operation: {}", result.operation_type);
        println!("  Total Operations: {}", result.total_operations);
        println!("  Success Rate: {:.2}%", result.success_rate * 100.0);
        println!("  Average Latency: {:.2?}", result.average_latency);
        println!("  Min Latency: {:.2?}", result.min_latency);
        println!("  Max Latency: {:.2?}", result.max_latency);
        println!("  Operations/sec: {:.2}", result.operations_per_second);
        println!("  Total Duration: {:.2?}", result.total_duration);
        println!();
    }

    println!("Summary");
    println!("-------");
    let total_ops: usize = suite.results.iter().map(|r| r.total_operations).sum();
    let avg_ops_per_sec: f64 = suite.results.iter().map(|r| r.operations_per_second).sum::<f64>() / suite.results.len() as f64;
    let avg_success_rate: f64 = suite.results.iter().map(|r| r.success_rate).sum::<f64>() / suite.results.len() as f64;

    println!("Total Operations: {total_ops}");
    println!("Average Ops/sec: {avg_ops_per_sec:.2}");
    println!("Average Success Rate: {:.2}%", avg_success_rate * 100.0);
}

fn print_csv_results(suite: &mem0_blueprint_lib::benchmarks::BenchmarkSuite) {
    println!("operation_type,total_operations,success_rate,avg_latency_ms,min_latency_ms,max_latency_ms,ops_per_sec,total_duration_ms");
    
    for result in &suite.results {
        println!("{},{},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
            result.operation_type,
            result.total_operations,
            result.success_rate,
            result.average_latency.as_secs_f64() * 1000.0,
            result.min_latency.as_secs_f64() * 1000.0,
            result.max_latency.as_secs_f64() * 1000.0,
            result.operations_per_second,
            result.total_duration.as_secs_f64() * 1000.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_conversion() {
        let simple: SearchComplexity = ComplexityArg::Simple.into();
        let medium: SearchComplexity = ComplexityArg::Medium.into();
        let complex: SearchComplexity = ComplexityArg::Complex.into();

        matches!(simple, SearchComplexity::Simple);
        matches!(medium, SearchComplexity::Medium);
        matches!(complex, SearchComplexity::Complex);
    }

    #[tokio::test]
    async fn test_benchmark_with_minimal_config() {
        let args = BenchmarkArgs {
            operations: 5,
            concurrent: 1,
            content_size: 10,
            complexity: ComplexityArg::Simple,
            delay_ms: 0,
            json_output: false,
            csv_output: false,
            output_file: None,
        };

        let result = run_benchmark(args).await;
        assert!(result.is_ok());
    }
}
