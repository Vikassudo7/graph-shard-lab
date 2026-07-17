use std::collections::{HashMap, HashSet};

struct User {
    id: u64,
    name: String,
}

struct Graph {
    users: HashMap<u64, User>,
    follows: HashMap<u64, Vec<u64>>,
}

impl Graph {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            follows: HashMap::new(),
        }
    }

    fn add_user(&mut self, id: u64, name: &str) {
        self.users.insert(
            id,
            User {
                id,
                name: name.to_string(),
            },
        );
    }

    fn add_follow(&mut self, source: u64, target: u64) -> Result<(), String> {
        if !self.users.contains_key(&source) {
            return Err(format!("Source user {source} does not exist"));
        }

        if !self.users.contains_key(&target) {
            return Err(format!("Target user {target} does not exist"));
        }

        let targets = self.follows.entry(source).or_default();

        if !targets.contains(&target) {
            targets.push(target);
        }

        Ok(())
    }

    fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    fn get_following_ids(&self, source: u64) -> &[u64] {
        match self.follows.get(&source) {
            Some(targets) => targets,
            None => &[],
        }
    }

    fn get_two_hop_ids(&self, source: u64) -> Vec<u64> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for first_hop in self.get_following_ids(source) {
            for second_hop in self.get_following_ids(*first_hop) {
                if *second_hop != source && seen.insert(*second_hop) {
                    result.push(*second_hop);
                }
            }
        }

        result
    }
}

fn build_sample_graph() -> Graph {
    let mut graph = Graph::new();

    graph.add_user(1, "Alice");
    graph.add_user(2, "Bob");
    graph.add_user(3, "Charlie");
    graph.add_user(4, "Diana");

    graph.add_follow(1, 2).expect("Failed to add edge");
    graph.add_follow(1, 3).expect("Failed to add edge");
    graph.add_follow(2, 3).expect("Failed to add edge");
    graph.add_follow(3, 4).expect("Failed to add edge");

    graph
}

fn main() {
    let graph = build_sample_graph();

    let alice = graph.get_user(1).expect("Alice should exist");

    println!("{} follows:", alice.name);

    for id in graph.get_following_ids(alice.id) {
        let user = graph.get_user(*id).expect("Referenced user should exist");
        println!("  {}", user.name);
    }

    println!("\nReachable from {} in exactly two hops:", alice.name);

    for id in graph.get_two_hop_ids(alice.id) {
        let user = graph.get_user(id).expect("Referenced user should exist");
        println!("  {}", user.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_one_hop_users() {
        let graph = build_sample_graph();

        assert_eq!(graph.get_following_ids(1), &[2, 3]);
    }

    #[test]
    fn returns_two_hop_users() {
        let graph = build_sample_graph();

        assert_eq!(graph.get_two_hop_ids(1), vec![3, 4]);
    }

    #[test]
    fn rejects_missing_target_user() {
        let mut graph = Graph::new();

        graph.add_user(1, "Alice");

        let result = graph.add_follow(1, 999);

        assert!(result.is_err());
    }

    #[test]
    fn does_not_duplicate_edges() {
        let mut graph = Graph::new();

        graph.add_user(1, "Alice");
        graph.add_user(2, "Bob");

        graph.add_follow(1, 2).unwrap();
        graph.add_follow(1, 2).unwrap();

        assert_eq!(graph.get_following_ids(1), &[2]);
    }
}
