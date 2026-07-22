use std::{fs, path::Path, time::Duration};

use graph_shard_lab::{
    distributed::DistributedShardedGraph,
    distributed_latency::{DistributedLatencyComparison, benchmark_two_hop_latencies},
    error::{GraphError, Result},
};

const SHARD_COUNT: usize = 4;
const CHANNEL_CAPACITY: usize = 256;
const QUERY_SOURCE_COUNT: u64 = 100;
const REPETITIONS: usize = 3;
const SIMULATED_DELAY_MS: u64 = 2;

fn duration_micros(duration: Duration) -> u128 {
    duration.as_micros()
}

fn reduction_percent(direct: Duration, batched: Duration) -> f64 {
    let direct_seconds = direct.as_secs_f64();

    if direct_seconds == 0.0 {
        return 0.0;
    }

    ((direct_seconds - batched.as_secs_f64()) / direct_seconds) * 100.0
}

fn write_results(path: &str, comparison: DistributedLatencyComparison) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|error| GraphError::IoError(error.to_string()))?;
    }

    let csv = format!(
        concat!(
            "strategy,samples,p50_us,p95_us,p99_us\n",
            "direct,{},{},{},{}\n",
            "batched,{},{},{},{}\n"
        ),
        comparison.direct.samples,
        duration_micros(comparison.direct.p50),
        duration_micros(comparison.direct.p95),
        duration_micros(comparison.direct.p99),
        comparison.batched.samples,
        duration_micros(comparison.batched.p50),
        duration_micros(comparison.batched.p95),
        duration_micros(comparison.batched.p99),
    );

    fs::write(path, csv).map_err(|error| GraphError::IoError(error.to_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut graph = DistributedShardedGraph::new_with_read_delay(
        SHARD_COUNT,
        CHANNEL_CAPACITY,
        Duration::from_millis(SIMULATED_DELAY_MS),
    )?;

    let mut query_sources = Vec::with_capacity(QUERY_SOURCE_COUNT as usize);

    for source_index in 0..QUERY_SOURCE_COUNT {
        let source = source_index + 1;

        graph.add_user(source, &format!("source-{source}")).await?;

        query_sources.push(source);

        /*
        Fanout cycles from one through eight.

        Sequential IDs spread first-hop users across the
        four hash shards.
        */
        let fanout = 1 + source_index % 8;

        for edge_index in 0..fanout {
            let first_hop = 10_000 + source_index * 16 + edge_index;

            let second_hop = 20_000 + source_index * 16 + edge_index;

            graph
                .add_user(first_hop, &format!("first-hop-{first_hop}"))
                .await?;

            graph
                .add_user(second_hop, &format!("second-hop-{second_hop}"))
                .await?;

            graph.add_follow(source, first_hop).await?;

            graph.add_follow(first_hop, second_hop).await?;
        }
    }

    let comparison = benchmark_two_hop_latencies(&graph, &query_sources, REPETITIONS).await?;

    println!("Distributed two-hop latency benchmark");
    println!(
        "Queries: {} sources × {} repetitions",
        query_sources.len(),
        REPETITIONS,
    );
    println!("Simulated read-message delay: {} ms", SIMULATED_DELAY_MS,);

    println!(
        "\nDirect  p50={}us p95={}us p99={}us",
        duration_micros(comparison.direct.p50),
        duration_micros(comparison.direct.p95),
        duration_micros(comparison.direct.p99),
    );

    println!(
        "Batched p50={}us p95={}us p99={}us",
        duration_micros(comparison.batched.p50),
        duration_micros(comparison.batched.p95),
        duration_micros(comparison.batched.p99),
    );

    println!(
        "\nReduction p50={:.2}% p95={:.2}% p99={:.2}%",
        reduction_percent(comparison.direct.p50, comparison.batched.p50,),
        reduction_percent(comparison.direct.p95, comparison.batched.p95,),
        reduction_percent(comparison.direct.p99, comparison.batched.p99,),
    );

    let output_path = "results/distributed_latency.csv";

    write_results(output_path, comparison)?;

    println!("\nSaved results to {output_path}");

    Ok(())
}
