use std::time::Instant;

use graph_shard_lab::sharded::ShardedGraph;

fn main() -> Result<(), String> {
    let user_count = 10_000;
    let edges_per_user = 8;
    let shard_count = 4;

    let started = Instant::now();

    let mut graph = ShardedGraph::new(shard_count)?;

    for id in 1..=user_count {
        graph.add_user(id, &format!("user-{id}"))?;
    }

    for source in 1..=user_count {
        for offset in 1..=edges_per_user {
            let target = ((source - 1 + offset) % user_count) + 1;
            graph.add_follow(source, target)?;
        }
    }

    println!(
        "Built graph with {} users, {} edges and {} shards in {:?}",
        graph.user_count(),
        graph.edge_count(),
        graph.shard_count(),
        started.elapsed()
    );

    println!("Users per shard: {:?}", graph.users_per_shard());

    let source = 1;
    let query_started = Instant::now();

    let result = graph.get_two_hop_with_stats(source);

    println!(
        "Two-hop query returned {} users in {:?}",
        result.user_ids.len(),
        query_started.elapsed()
    );

    println!("Shards touched: {}", result.shards_touched);
    println!("Cross-shard hops: {}", result.cross_shard_hops);

    Ok(())
}
