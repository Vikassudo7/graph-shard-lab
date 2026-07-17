use std::collections::HashSet;

use crate::{Graph, User};

#[derive(Debug)]
pub struct QueryResult {
    pub user_ids: Vec<u64>,
    pub shards_touched: usize,
    pub cross_shard_hops: usize,
}

pub struct ShardedGraph {
    shards: Vec<Graph>,
}

impl ShardedGraph {
    pub fn new(shard_count: usize) -> Result<Self, String> {
        if shard_count == 0 {
            return Err("Shard count must be greater than zero".to_string());
        }

        let mut shards = Vec::with_capacity(shard_count);

        for _ in 0..shard_count {
            shards.push(Graph::new());
        }

        Ok(Self { shards })
    }

    fn shard_for(&self, user_id: u64) -> usize {
        user_id as usize % self.shards.len()
    }

    pub fn add_user(&mut self, id: u64, name: &str) -> Result<(), String> {
        let shard_id = self.shard_for(id);
        self.shards[shard_id].add_user(id, name)
    }

    pub fn add_follow(&mut self, source: u64, target: u64) -> Result<(), String> {
        if self.get_user(source).is_none() {
            return Err(format!("Source user {source} does not exist"));
        }

        if self.get_user(target).is_none() {
            return Err(format!("Target user {target} does not exist"));
        }

        let source_shard = self.shard_for(source);

        self.shards[source_shard].add_follow_unchecked(source, target)
    }

    pub fn get_user(&self, id: u64) -> Option<&User> {
        let shard_id = self.shard_for(id);
        self.shards[shard_id].get_user(id)
    }

    pub fn get_following_ids(&self, source: u64) -> &[u64] {
        let shard_id = self.shard_for(source);
        self.shards[shard_id].get_following_ids(source)
    }

    pub fn get_two_hop_with_stats(&self, source: u64) -> QueryResult {
        let source_shard = self.shard_for(source);

        let mut user_ids = Vec::new();
        let mut seen_users = HashSet::new();
        let mut touched_shards = HashSet::new();
        let mut cross_shard_hops = 0;

        touched_shards.insert(source_shard);

        for first_hop in self.get_following_ids(source) {
            let first_hop_shard = self.shard_for(*first_hop);

            touched_shards.insert(first_hop_shard);

            if first_hop_shard != source_shard {
                cross_shard_hops += 1;
            }

            for second_hop in self.get_following_ids(*first_hop) {
                let second_hop_shard = self.shard_for(*second_hop);

                touched_shards.insert(second_hop_shard);

                if second_hop_shard != first_hop_shard {
                    cross_shard_hops += 1;
                }

                if *second_hop != source && seen_users.insert(*second_hop) {
                    user_ids.push(*second_hop);
                }
            }
        }

        QueryResult {
            user_ids,
            shards_touched: touched_shards.len(),
            cross_shard_hops,
        }
    }

    pub fn get_two_hop_ids(&self, source: u64) -> Vec<u64> {
        self.get_two_hop_with_stats(source).user_ids
    }

    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }

    pub fn user_count(&self) -> usize {
        self.shards.iter().map(Graph::user_count).sum()
    }

    pub fn edge_count(&self) -> usize {
        self.shards.iter().map(Graph::edge_count).sum()
    }

    pub fn users_per_shard(&self) -> Vec<usize> {
        self.shards.iter().map(Graph::user_count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn places_users_across_shards() {
        let mut graph = ShardedGraph::new(4).unwrap();

        for id in 1..=8 {
            graph.add_user(id, &format!("user-{id}")).unwrap();
        }

        assert_eq!(graph.user_count(), 8);
        assert_eq!(graph.shard_count(), 4);
        assert_eq!(graph.users_per_shard(), vec![2, 2, 2, 2]);
    }

    #[test]
    fn supports_cross_shard_edges() {
        let mut graph = ShardedGraph::new(4).unwrap();

        graph.add_user(1, "Alice").unwrap();
        graph.add_user(2, "Bob").unwrap();

        graph.add_follow(1, 2).unwrap();

        assert_eq!(graph.get_following_ids(1), &[2]);
    }

    #[test]
    fn supports_cross_shard_two_hop_queries() {
        let mut graph = ShardedGraph::new(4).unwrap();

        graph.add_user(1, "Alice").unwrap();
        graph.add_user(2, "Bob").unwrap();
        graph.add_user(3, "Charlie").unwrap();
        graph.add_user(4, "Diana").unwrap();

        graph.add_follow(1, 2).unwrap();
        graph.add_follow(2, 3).unwrap();
        graph.add_follow(2, 4).unwrap();

        assert_eq!(graph.get_two_hop_ids(1), vec![3, 4]);
    }

    #[test]
    fn records_query_statistics() {
        let mut graph = ShardedGraph::new(4).unwrap();

        graph.add_user(1, "Alice").unwrap();
        graph.add_user(2, "Bob").unwrap();
        graph.add_user(3, "Charlie").unwrap();

        graph.add_follow(1, 2).unwrap();
        graph.add_follow(2, 3).unwrap();

        let result = graph.get_two_hop_with_stats(1);

        assert_eq!(result.user_ids, vec![3]);
        assert_eq!(result.shards_touched, 3);
        assert_eq!(result.cross_shard_hops, 2);
    }
}
