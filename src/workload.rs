use crate::Graph;

pub fn build_uniform_graph(user_count: u64, edges_per_user: u64) -> Result<Graph, String> {
    if user_count == 0 {
        return Err("User count must be greater than zero".to_string());
    }

    if edges_per_user >= user_count {
        return Err("Edges per user must be smaller than user count".to_string());
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
