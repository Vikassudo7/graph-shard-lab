use std::time::Duration;

use tokio::time::Instant;

use crate::distributed::DistributedShardedGraph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencySummary {
    pub samples: usize,
    pub p50: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistributedLatencyComparison {
    pub direct: LatencySummary,
    pub batched: LatencySummary,
}

fn nearest_rank(sorted_samples: &[Duration], percentile: usize) -> Duration {
    let rank = percentile
        .saturating_mul(sorted_samples.len())
        .div_ceil(100);

    sorted_samples[rank.saturating_sub(1)]
}

pub fn summarize_latencies(samples: &[Duration]) -> Result<LatencySummary, String> {
    if samples.is_empty() {
        return Err("At least one latency sample is required".to_string());
    }

    let mut sorted_samples = samples.to_vec();
    sorted_samples.sort_unstable();

    Ok(LatencySummary {
        samples: sorted_samples.len(),
        p50: nearest_rank(&sorted_samples, 50),
        p95: nearest_rank(&sorted_samples, 95),
        p99: nearest_rank(&sorted_samples, 99),
    })
}

pub async fn benchmark_two_hop_latencies(
    graph: &DistributedShardedGraph,
    sources: &[u64],
    repetitions: usize,
) -> Result<DistributedLatencyComparison, String> {
    if sources.is_empty() {
        return Err("At least one query source is required".to_string());
    }

    if repetitions == 0 {
        return Err("Benchmark repetitions must be greater than zero".to_string());
    }

    let sample_capacity = sources
        .len()
        .checked_mul(repetitions)
        .ok_or_else(|| "Latency sample count is too large".to_string())?;

    let mut direct_samples = Vec::with_capacity(sample_capacity);

    let mut batched_samples = Vec::with_capacity(sample_capacity);

    for repetition in 0..repetitions {
        for (source_index, source) in sources.iter().copied().enumerate() {
            /*
            Alternate execution order to reduce systematic bias
            from always running one strategy first.
            */
            if (repetition + source_index) % 2 == 0 {
                let direct_start = Instant::now();

                graph.get_two_hop(source).await?;

                direct_samples.push(direct_start.elapsed());

                let batched_start = Instant::now();

                graph.get_two_hop_batched(source).await?;

                batched_samples.push(batched_start.elapsed());
            } else {
                let batched_start = Instant::now();

                graph.get_two_hop_batched(source).await?;

                batched_samples.push(batched_start.elapsed());

                let direct_start = Instant::now();

                graph.get_two_hop(source).await?;

                direct_samples.push(direct_start.elapsed());
            }
        }
    }

    Ok(DistributedLatencyComparison {
        direct: summarize_latencies(&direct_samples)?,
        batched: summarize_latencies(&batched_samples)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_summary_uses_nearest_rank_percentiles() {
        let samples: Vec<Duration> = (1..=100).map(Duration::from_millis).collect();

        let summary = summarize_latencies(&samples).unwrap();

        assert_eq!(summary.samples, 100);
        assert_eq!(summary.p50, Duration::from_millis(50));
        assert_eq!(summary.p95, Duration::from_millis(95));
        assert_eq!(summary.p99, Duration::from_millis(99));
    }

    #[test]
    fn latency_summary_rejects_empty_samples() {
        assert!(summarize_latencies(&[]).is_err());
    }

    #[tokio::test(start_paused = true)]
    async fn distributed_benchmark_reports_lower_batched_tail() {
        let mut graph =
            DistributedShardedGraph::new_with_read_delay(4, 32, Duration::from_millis(10)).unwrap();

        for id in 1..=11 {
            graph.add_user(id, &format!("user-{id}")).await.unwrap();
        }

        // Four first-hop users spread across four shards.
        graph.add_follow(1, 4).await.unwrap();
        graph.add_follow(1, 5).await.unwrap();
        graph.add_follow(1, 6).await.unwrap();
        graph.add_follow(1, 7).await.unwrap();

        graph.add_follow(4, 8).await.unwrap();
        graph.add_follow(5, 9).await.unwrap();
        graph.add_follow(6, 10).await.unwrap();
        graph.add_follow(7, 11).await.unwrap();

        let comparison = benchmark_two_hop_latencies(&graph, &[1], 3).await.unwrap();

        assert_eq!(comparison.direct.samples, 3);
        assert_eq!(comparison.batched.samples, 3);

        /*
        Direct:
        source read + four sequential first-hop reads
        = five delay rounds.
        */
        assert_eq!(comparison.direct.p99, Duration::from_millis(50),);

        /*
        Batched:
        source read + one concurrent batch round
        = two delay rounds.
        */
        assert_eq!(comparison.batched.p99, Duration::from_millis(20),);

        assert!(comparison.batched.p99 < comparison.direct.p99);
    }

    #[tokio::test]
    async fn distributed_benchmark_rejects_invalid_input() {
        let graph = DistributedShardedGraph::new(2, 32).unwrap();

        assert!(benchmark_two_hop_latencies(&graph, &[], 1).await.is_err());

        assert!(benchmark_two_hop_latencies(&graph, &[1], 0).await.is_err());
    }
}
