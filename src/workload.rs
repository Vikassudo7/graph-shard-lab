use crate::Graph;
use crate::error::{GraphError, Result};

pub fn build_uniform_graph(user_count: u64, edges_per_user: u64) -> Result<Graph> {
    if user_count == 0 {
        return Err(GraphError::ZeroUserCount);
    }

    if edges_per_user >= user_count {
        return Err(GraphError::EdgesPerUserTooLarge);
    }

    let mut graph = Graph::new();

    for id in 1..=user_count {
        graph.add_user(id, &format!("user-{id}"))?;
    }

    for source in 1..=user_count {
        for offset in 1..=edges_per_user {
            let target = ((source - 1 + offset) % user_count) + 1;
            graph.add_follow(source, target)?;
        }
    }

    Ok(graph)
}

use rand::{Rng, SeedableRng, rngs::StdRng};

#[derive(Debug)]
pub struct CommunityWorkload {
    pub user_count: u64,
    pub community_count: u64,
    pub edges: Vec<(u64, u64)>,
}

#[derive(Debug)]
pub struct HubWorkload {
    pub user_count: u64,
    pub hub_ids: Vec<u64>,
    pub edges: Vec<(u64, u64)>,
}

pub fn generate_community_workload(
    user_count: u64,
    community_count: u64,
    edges_per_user: u64,
    local_edges_per_user: u64,
    seed: u64,
) -> Result<CommunityWorkload> {
    if user_count == 0 {
        return Err(GraphError::ZeroUserCount);
    }

    if community_count == 0 {
        return Err(GraphError::EmptyCommunities);
    }

    if !user_count.is_multiple_of(community_count) {
        return Err(GraphError::UserCountNotDivisible);
    }

    if local_edges_per_user > edges_per_user {
        return Err(GraphError::LocalEdgesExceedTotal);
    }

    let community_size = user_count / community_count;

    if local_edges_per_user >= community_size {
        return Err(GraphError::LocalEdgesExceedCommunitySize);
    }

    let cross_edges_per_user = edges_per_user - local_edges_per_user;

    let users_outside_community = user_count - community_size;

    if cross_edges_per_user > users_outside_community {
        return Err(GraphError::CrossEdgesExceedExternal);
    }

    let mut rng = StdRng::seed_from_u64(seed);

    let mut edges = Vec::with_capacity((user_count * edges_per_user) as usize);

    for source in 1..=user_count {
        let community_id = (source - 1) / community_size;

        let community_start = community_id * community_size + 1;

        let community_end = community_start + community_size - 1;

        let mut targets = std::collections::HashSet::new();

        while targets.len() < local_edges_per_user as usize {
            let target = rng.gen_range(community_start..=community_end);

            if target != source {
                targets.insert(target);
            }
        }

        while targets.len() < edges_per_user as usize {
            let target = rng.gen_range(1..=user_count);

            let target_community = (target - 1) / community_size;

            if target != source && target_community != community_id {
                targets.insert(target);
            }
        }

        let mut ordered_targets: Vec<u64> = targets.into_iter().collect();

        ordered_targets.sort_unstable();

        for target in ordered_targets {
            edges.push((source, target));
        }
    }

    Ok(CommunityWorkload {
        user_count,
        community_count,
        edges,
    })
}
pub fn generate_hub_workload(
    user_count: u64,
    hub_count: u64,
    edges_per_user: u64,
    hub_edges_per_user: u64,
    seed: u64,
) -> Result<HubWorkload> {
    if user_count == 0 {
        return Err(GraphError::ZeroUserCount);
    }

    if hub_count == 0 {
        return Err(GraphError::ZeroHubCount);
    }

    if hub_count >= user_count {
        return Err(GraphError::HubCountTooLarge);
    }

    if edges_per_user >= user_count {
        return Err(GraphError::EdgesPerUserTooLarge);
    }

    if hub_edges_per_user > edges_per_user {
        return Err(GraphError::HubEdgesExceedTotal);
    }

    // Hub users cannot follow themselves, so a hub has only
    // hub_count - 1 possible hub targets.
    if hub_edges_per_user >= hub_count {
        return Err(GraphError::TooManyHubEdges);
    }

    let regular_edges_per_user = edges_per_user - hub_edges_per_user;
    let regular_user_count = user_count - hub_count;

    // A normal user cannot follow itself, so it has at most
    // regular_user_count - 1 distinct normal targets.
    if regular_edges_per_user >= regular_user_count {
        return Err(GraphError::TooManyRegularEdges);
    }

    let hub_ids: Vec<u64> = (1..=hub_count).collect();

    let mut rng = StdRng::seed_from_u64(seed);

    let mut edges = Vec::with_capacity((user_count * edges_per_user) as usize);

    for source in 1..=user_count {
        let mut targets = std::collections::HashSet::new();

        // Select the requested number of popular hub targets.
        while targets.len() < hub_edges_per_user as usize {
            let target = rng.gen_range(1..=hub_count);

            if target != source {
                targets.insert(target);
            }
        }

        // Fill the remaining edges using normal, non-hub users.
        while targets.len() < edges_per_user as usize {
            let target = rng.gen_range((hub_count + 1)..=user_count);

            if target != source {
                targets.insert(target);
            }
        }

        let mut ordered_targets: Vec<u64> = targets.into_iter().collect();

        ordered_targets.sort_unstable();

        for target in ordered_targets {
            edges.push((source, target));
        }
    }

    Ok(HubWorkload {
        user_count,
        hub_ids,
        edges,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_expected_number_of_users_and_edges() {
        let graph = build_uniform_graph(100, 4).unwrap();

        assert_eq!(graph.user_count(), 100);
        assert_eq!(graph.edge_count(), 400);
    }

    #[test]
    fn creates_deterministic_edges() {
        let graph = build_uniform_graph(100, 4).unwrap();

        assert_eq!(graph.get_following_ids(1), &[2, 3, 4, 5]);
        assert_eq!(graph.get_following_ids(99), &[100, 1, 2, 3]);
    }

    #[test]
    fn rejects_invalid_workloads() {
        assert!(build_uniform_graph(0, 0).is_err());
        assert!(build_uniform_graph(10, 10).is_err());
    }
}
#[test]
fn seeded_community_workload_is_repeatable() {
    let first = generate_community_workload(100, 10, 8, 6, 42).unwrap();

    let second = generate_community_workload(100, 10, 8, 6, 42).unwrap();

    assert_eq!(first.edges, second.edges);
}

#[test]
fn different_seeds_generate_different_edges() {
    let first = generate_community_workload(100, 10, 8, 6, 42).unwrap();

    let second = generate_community_workload(100, 10, 8, 6, 99).unwrap();

    assert_ne!(first.edges, second.edges);
}

#[test]
fn generates_requested_number_of_edges() {
    let workload = generate_community_workload(100, 10, 8, 6, 42).unwrap();

    assert_eq!(workload.edges.len(), 800);
}

#[test]
fn generates_requested_local_edge_ratio() {
    let workload = generate_community_workload(100, 10, 8, 6, 42).unwrap();

    let community_size = 10;

    for source in 1..=100 {
        let source_community = (source - 1) / community_size;

        let source_edges: Vec<_> = workload
            .edges
            .iter()
            .filter(|(edge_source, _)| *edge_source == source)
            .collect();

        let local_count = source_edges
            .iter()
            .filter(|(_, target)| (*target - 1) / community_size == source_community)
            .count();

        assert_eq!(source_edges.len(), 8);
        assert_eq!(local_count, 6);
    }
}

#[test]
fn seeded_hub_workload_is_repeatable() {
    let first = generate_hub_workload(100, 5, 8, 2, 42).unwrap();

    let second = generate_hub_workload(100, 5, 8, 2, 42).unwrap();

    assert_eq!(first.edges, second.edges);
    assert_eq!(first.hub_ids, second.hub_ids);
}

#[test]
fn hub_workload_generates_requested_edges() {
    let workload = generate_hub_workload(100, 5, 8, 2, 42).unwrap();

    assert_eq!(workload.user_count, 100);
    assert_eq!(workload.hub_ids, vec![1, 2, 3, 4, 5]);
    assert_eq!(workload.edges.len(), 800);

    for source in 1..=100 {
        let source_edges: Vec<_> = workload
            .edges
            .iter()
            .filter(|(edge_source, _)| *edge_source == source)
            .collect();

        let hub_target_count = source_edges
            .iter()
            .filter(|(_, target)| *target <= 5)
            .count();

        assert_eq!(source_edges.len(), 8);
        assert_eq!(hub_target_count, 2);
    }
}

#[test]
fn hub_workload_rejects_invalid_parameters() {
    assert!(generate_hub_workload(0, 5, 8, 2, 42).is_err());
    assert!(generate_hub_workload(100, 0, 8, 2, 42).is_err());
    assert!(generate_hub_workload(100, 100, 8, 2, 42).is_err());
    assert!(generate_hub_workload(100, 5, 8, 9, 42).is_err());
}
