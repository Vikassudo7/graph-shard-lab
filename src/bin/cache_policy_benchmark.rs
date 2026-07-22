use graph_shard_lab::cache::{AdjacencyLruCache, EvictionPolicy};
use std::fs;

const SHARD_COUNT: usize = 4;
const USER_COUNT: u64 = 10_000;
const HUB_COUNT: u64 = 100;
const TOTAL_ACCESSES: usize = 80_000;
const EDGES_PER_USER: u64 = 8;

const CAPACITIES_PER_SHARD: [usize; 4] = [25, 50, 100, 250];

#[derive(Debug)]
struct RunStats {
    hits: usize,
    misses: usize,
}

impl RunStats {
    fn total_accesses(&self) -> usize {
        self.hits + self.misses
    }

    fn hit_rate_percent(&self) -> f64 {
        let total = self.total_accesses();

        if total == 0 {
            0.0
        } else {
            self.hits as f64 * 100.0 / total as f64
        }
    }
}

fn policy_name(policy: EvictionPolicy) -> &'static str {
    match policy {
        EvictionPolicy::Lru => "lru",
        EvictionPolicy::Fifo => "fifo",
        EvictionPolicy::Lfu => "lfu",
    }
}

fn shard_for(user_id: u64) -> usize {
    user_id as usize % SHARD_COUNT
}

fn adjacency_for(user_id: u64) -> Vec<u64> {
    (1..=EDGES_PER_USER)
        .map(|offset| ((user_id - 1 + offset) % USER_COUNT) + 1)
        .collect()
}

fn build_access_trace() -> Vec<u64> {
    let mut trace = Vec::with_capacity(TOTAL_ACCESSES);

    for access_index in 0..TOTAL_ACCESSES {
        let user_id = if access_index % 4 == 0 {
            // Exactly 25% of accesses target the 100 hub users.
            1 + ((access_index / 4) as u64 % HUB_COUNT)
        } else {
            // Remaining accesses rotate through the normal-user population.
            HUB_COUNT + 1 + ((access_index as u64 * 17) % (USER_COUNT - HUB_COUNT))
        };

        trace.push(user_id);
    }

    trace
}

fn build_caches(
    capacity_per_shard: usize,
    policy: EvictionPolicy,
) -> Result<Vec<AdjacencyLruCache>, String> {
    (0..SHARD_COUNT)
        .map(|_| AdjacencyLruCache::new_with_policy(capacity_per_shard, policy))
        .collect()
}

fn warm_hubs(caches: &mut [AdjacencyLruCache]) {
    for user_id in 1..=HUB_COUNT {
        let shard_id = shard_for(user_id);
        let adjacency = adjacency_for(user_id);

        let _ = caches[shard_id].insert(user_id, adjacency);
    }
}

fn run_trace(
    trace: &[u64],
    capacity_per_shard: usize,
    policy: EvictionPolicy,
    warmed: bool,
) -> Result<RunStats, String> {
    let mut caches = build_caches(capacity_per_shard, policy)?;

    if warmed {
        warm_hubs(&mut caches);
    }

    let mut hits = 0;
    let mut misses = 0;

    for &user_id in trace {
        let shard_id = shard_for(user_id);
        let expected = adjacency_for(user_id);

        match caches[shard_id].get(user_id) {
            Some(cached) => {
                if cached != expected {
                    return Err(format!("Incorrect cached adjacency for user {user_id}"));
                }

                hits += 1;
            }

            None => {
                misses += 1;
                let _ = caches[shard_id].insert(user_id, expected);
            }
        }
    }

    Ok(RunStats { hits, misses })
}

fn main() -> Result<(), String> {
    let trace = build_access_trace();

    let policies = [
        EvictionPolicy::Lru,
        EvictionPolicy::Fifo,
        EvictionPolicy::Lfu,
    ];

    let mut csv_rows = vec![
        "policy,mode,capacity_per_shard,total_capacity,total_accesses,\
         cache_hits,cache_misses,hit_rate_percent"
            .replace(' ', ""),
    ];

    println!(
        "{:<6} {:<7} {:>10} {:>10} {:>10} {:>10}",
        "Policy", "Mode", "Capacity", "Hits", "Misses", "Hit rate"
    );

    println!("{}", "-".repeat(64));

    for policy in policies {
        for capacity_per_shard in CAPACITIES_PER_SHARD {
            for warmed in [false, true] {
                let stats = run_trace(&trace, capacity_per_shard, policy, warmed)?;

                let mode = if warmed { "warmed" } else { "cold" };
                let total_capacity = capacity_per_shard * SHARD_COUNT;

                println!(
                    "{:<6} {:<7} {:>10} {:>10} {:>10} {:>9.2}%",
                    policy_name(policy),
                    mode,
                    total_capacity,
                    stats.hits,
                    stats.misses,
                    stats.hit_rate_percent(),
                );

                csv_rows.push(format!(
                    "{},{},{},{},{},{},{},{:.4}",
                    policy_name(policy),
                    mode,
                    capacity_per_shard,
                    total_capacity,
                    stats.total_accesses(),
                    stats.hits,
                    stats.misses,
                    stats.hit_rate_percent(),
                ));
            }
        }
    }

    fs::create_dir_all("results")
        .map_err(|error| format!("Failed to create results directory: {error}"))?;

    fs::write(
        "results/cache_policy_benchmark.csv",
        csv_rows.join("\n") + "\n",
    )
    .map_err(|error| format!("Failed to write benchmark CSV: {error}"))?;

    println!();
    println!("Saved results/cache_policy_benchmark.csv");

    Ok(())
}
