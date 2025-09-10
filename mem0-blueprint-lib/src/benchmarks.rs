use crate::{AddMemoryRequest, SearchMemoryRequest, UpdateMemoryRequest, MemoryContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub num_operations: usize,
    pub concurrent_operations: usize,
    pub memory_content_size: usize,
    pub search_query_complexity: SearchComplexity,
    pub delay_between_operations: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchComplexity {
    Simple,
    Medium,
    Complex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub operation_type: String,
    pub total_operations: usize,
    pub total_duration: Duration,
    pub average_latency: Duration,
    pub min_latency: Duration,
    pub max_latency: Duration,
    pub operations_per_second: f64,
    pub success_rate: f64,
    pub memory_usage_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub config: BenchmarkConfig,
    pub results: Vec<BenchmarkResult>,
    pub total_duration: Duration,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            num_operations: 1000,
            concurrent_operations: 10,
            memory_content_size: 100,
            search_query_complexity: SearchComplexity::Medium,
            delay_between_operations: Duration::from_millis(1),
        }
    }
}

pub struct MemoryBenchmark {
    context: MemoryContext,
    config: BenchmarkConfig,
}

impl MemoryBenchmark {
    pub fn new(config: BenchmarkConfig) -> Self {
        Self {
            context: MemoryContext::new(),
            config,
        }
    }

    pub async fn run_full_benchmark(&self) -> BenchmarkSuite {
        let start_time = Instant::now();
        let mut results = Vec::new();

        results.push(self.benchmark_add_memory().await);
        results.push(self.benchmark_search_memory().await);
        results.push(self.benchmark_get_memory().await);
        results.push(self.benchmark_update_memory().await);
        results.push(self.benchmark_delete_memory().await);
        results.push(self.benchmark_mixed_operations().await);

        BenchmarkSuite {
            config: self.config.clone(),
            results,
            total_duration: start_time.elapsed(),
        }
    }

    pub async fn benchmark_add_memory(&self) -> BenchmarkResult {
        let mut latencies = Vec::new();
        let mut successful_operations = 0;

        let content = "A".repeat(self.config.memory_content_size);

        for i in 0..self.config.num_operations {
            let request = AddMemoryRequest {
                content: format!("{content} - Memory {i}"),
                user_id: Some(format!("user_{}", i % 100)),
                agent_id: Some(format!("agent_{}", i % 10)),
                session_id: Some(format!("session_{}", i % 50)),
                metadata: Some(HashMap::from([
                    ("benchmark".to_string(), "true".to_string()),
                    ("iteration".to_string(), i.to_string()),
                ])),
            };

            let start = Instant::now();
            let _memory = self.context.add_memory(request).await;
            let latency = start.elapsed();

            latencies.push(latency);
            successful_operations += 1;

            if self.config.delay_between_operations > Duration::ZERO {
                sleep(self.config.delay_between_operations).await;
            }
        }

        self.calculate_benchmark_result("add_memory", latencies, successful_operations)
    }

    pub async fn benchmark_search_memory(&self) -> BenchmarkResult {
        self.populate_test_data().await;

        let mut latencies = Vec::new();
        let mut successful_operations = 0;

        let queries = match self.config.search_query_complexity {
            SearchComplexity::Simple => vec!["Memory", "test", "data"],
            SearchComplexity::Medium => vec!["Memory 1", "test data", "benchmark true"],
            SearchComplexity::Complex => vec!["Memory 1 test", "benchmark true data", "complex search query"],
        };

        for i in 0..self.config.num_operations {
            let query = queries[i % queries.len()].to_string();
            let request = SearchMemoryRequest {
                query,
                user_id: Some(format!("user_{}", i % 100)),
                agent_id: None,
                session_id: None,
                limit: Some(10),
            };

            let start = Instant::now();
            let _memories = self.context.search_memories(request).await;
            let latency = start.elapsed();

            latencies.push(latency);
            successful_operations += 1;

            if self.config.delay_between_operations > Duration::ZERO {
                sleep(self.config.delay_between_operations).await;
            }
        }

        self.calculate_benchmark_result("search_memory", latencies, successful_operations)
    }

    pub async fn benchmark_get_memory(&self) -> BenchmarkResult {
        let memory_ids = self.populate_test_data().await;

        let mut latencies = Vec::new();
        let mut successful_operations = 0;

        for i in 0..self.config.num_operations {
            let memory_id = &memory_ids[i % memory_ids.len()];

            let start = Instant::now();
            let _memory = self.context.get_memory(memory_id).await;
            let latency = start.elapsed();

            latencies.push(latency);
            successful_operations += 1;

            if self.config.delay_between_operations > Duration::ZERO {
                sleep(self.config.delay_between_operations).await;
            }
        }

        self.calculate_benchmark_result("get_memory", latencies, successful_operations)
    }

    pub async fn benchmark_update_memory(&self) -> BenchmarkResult {
        let memory_ids = self.populate_test_data().await;

        let mut latencies = Vec::new();
        let mut successful_operations = 0;

        for i in 0..self.config.num_operations {
            let memory_id = memory_ids[i % memory_ids.len()].clone();
            let request = UpdateMemoryRequest {
                memory_id,
                content: format!("Updated content {i}"),
                metadata: Some(HashMap::from([
                    ("updated".to_string(), "true".to_string()),
                    ("iteration".to_string(), i.to_string()),
                ])),
            };

            let start = Instant::now();
            let _result = self.context.update_memory(request).await;
            let latency = start.elapsed();

            latencies.push(latency);
            successful_operations += 1;

            if self.config.delay_between_operations > Duration::ZERO {
                sleep(self.config.delay_between_operations).await;
            }
        }

        self.calculate_benchmark_result("update_memory", latencies, successful_operations)
    }

    pub async fn benchmark_delete_memory(&self) -> BenchmarkResult {
        let memory_ids = self.populate_test_data().await;

        let mut latencies = Vec::new();
        let mut successful_operations = 0;

        let delete_count = (memory_ids.len() / 2).min(self.config.num_operations);

        for i in 0..delete_count {
            let memory_id = &memory_ids[i];

            let start = Instant::now();
            let _result = self.context.delete_memory(memory_id).await;
            let latency = start.elapsed();

            latencies.push(latency);
            successful_operations += 1;

            if self.config.delay_between_operations > Duration::ZERO {
                sleep(self.config.delay_between_operations).await;
            }
        }

        self.calculate_benchmark_result("delete_memory", latencies, successful_operations)
    }

    pub async fn benchmark_mixed_operations(&self) -> BenchmarkResult {
        let mut latencies = Vec::new();
        let mut successful_operations = 0;

        for i in 0..self.config.num_operations {
            let start = Instant::now();

            match i % 4 {
                0 => {
                    let request = AddMemoryRequest {
                        content: format!("Mixed operation memory {i}"),
                        user_id: Some(format!("user_{}", i % 50)),
                        agent_id: None,
                        session_id: None,
                        metadata: None,
                    };
                    self.context.add_memory(request).await;
                }
                1 => {
                    let request = SearchMemoryRequest {
                        query: "Mixed".to_string(),
                        user_id: None,
                        agent_id: None,
                        session_id: None,
                        limit: Some(5),
                    };
                    self.context.search_memories(request).await;
                }
                2 => {
                    let request = SearchMemoryRequest {
                        query: "operation".to_string(),
                        user_id: Some(format!("user_{}", i % 50)),
                        agent_id: None,
                        session_id: None,
                        limit: Some(3),
                    };
                    self.context.search_memories(request).await;
                }
                _ => {
                    let request = SearchMemoryRequest {
                        query: "memory".to_string(),
                        user_id: None,
                        agent_id: None,
                        session_id: None,
                        limit: Some(1),
                    };
                    self.context.search_memories(request).await;
                }
            }

            let latency = start.elapsed();
            latencies.push(latency);
            successful_operations += 1;

            if self.config.delay_between_operations > Duration::ZERO {
                sleep(self.config.delay_between_operations).await;
            }
        }

        self.calculate_benchmark_result("mixed_operations", latencies, successful_operations)
    }

    async fn populate_test_data(&self) -> Vec<String> {
        let mut memory_ids = Vec::new();
        let test_data_count = 100.min(self.config.num_operations);

        for i in 0..test_data_count {
            let request = AddMemoryRequest {
                content: format!("Test data memory {i}"),
                user_id: Some(format!("user_{}", i % 20)),
                agent_id: Some(format!("agent_{}", i % 5)),
                session_id: Some(format!("session_{}", i % 10)),
                metadata: Some(HashMap::from([
                    ("test".to_string(), "true".to_string()),
                    ("index".to_string(), i.to_string()),
                ])),
            };

            let memory = self.context.add_memory(request).await;
            memory_ids.push(memory.id);
        }

        memory_ids
    }

    fn calculate_benchmark_result(
        &self,
        operation_type: &str,
        latencies: Vec<Duration>,
        successful_operations: usize,
    ) -> BenchmarkResult {
        let total_duration: Duration = latencies.iter().sum();
        let min_latency = latencies.iter().min().copied().unwrap_or_default();
        let max_latency = latencies.iter().max().copied().unwrap_or_default();
        let average_latency = if latencies.is_empty() {
            Duration::ZERO
        } else {
            total_duration / latencies.len() as u32
        };

        let operations_per_second = if total_duration.as_secs_f64() > 0.0 {
            successful_operations as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        let success_rate = if self.config.num_operations > 0 {
            successful_operations as f64 / self.config.num_operations as f64
        } else {
            0.0
        };

        BenchmarkResult {
            operation_type: operation_type.to_string(),
            total_operations: successful_operations,
            total_duration,
            average_latency,
            min_latency,
            max_latency,
            operations_per_second,
            success_rate,
            memory_usage_mb: 0.0, // Would need system monitoring for actual memory usage
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_benchmark_add_memory() {
        let config = BenchmarkConfig {
            num_operations: 10,
            concurrent_operations: 1,
            memory_content_size: 50,
            search_query_complexity: SearchComplexity::Simple,
            delay_between_operations: Duration::ZERO,
        };

        let benchmark = MemoryBenchmark::new(config);
        let result = benchmark.benchmark_add_memory().await;

        assert_eq!(result.operation_type, "add_memory");
        assert_eq!(result.total_operations, 10);
        assert_eq!(result.success_rate, 1.0);
        assert!(result.operations_per_second > 0.0);
    }

    #[tokio::test]
    async fn test_benchmark_search_memory() {
        let config = BenchmarkConfig {
            num_operations: 5,
            concurrent_operations: 1,
            memory_content_size: 50,
            search_query_complexity: SearchComplexity::Simple,
            delay_between_operations: Duration::ZERO,
        };

        let benchmark = MemoryBenchmark::new(config);
        let result = benchmark.benchmark_search_memory().await;

        assert_eq!(result.operation_type, "search_memory");
        assert_eq!(result.total_operations, 5);
        assert_eq!(result.success_rate, 1.0);
    }

    #[tokio::test]
    async fn test_full_benchmark_suite() {
        let config = BenchmarkConfig {
            num_operations: 3,
            concurrent_operations: 1,
            memory_content_size: 20,
            search_query_complexity: SearchComplexity::Simple,
            delay_between_operations: Duration::ZERO,
        };

        let benchmark = MemoryBenchmark::new(config);
        let suite = benchmark.run_full_benchmark().await;

        assert_eq!(suite.results.len(), 6);
        assert!(suite.total_duration > Duration::ZERO);

        for result in suite.results {
            assert!(result.success_rate > 0.0);
            assert!(result.total_operations > 0);
        }
    }
}
