use graph_shard_lab::Graph;

fn main() -> Result<(), String> {
    let mut graph = Graph::new();

    graph.add_user(1, "Alice")?;
    graph.add_user(2, "Bob")?;
    graph.add_user(3, "Charlie")?;
    graph.add_user(4, "Diana")?;

    graph.add_follow(1, 2)?;
    graph.add_follow(1, 3)?;
    graph.add_follow(2, 3)?;
    graph.add_follow(3, 4)?;

    let alice = graph.get_user(1).ok_or("Alice should exist")?;

    println!(
        "Graph contains {} users and {} edges",
        graph.user_count(),
        graph.edge_count()
    );

    println!("\n{} follows:", alice.name);

    for id in graph.get_following_ids(alice.id) {
        let user = graph.get_user(*id).ok_or("Referenced user is missing")?;
        println!("  {}", user.name);
    }

    println!("\nReachable from {} in exactly two hops:", alice.name);

    for id in graph.get_two_hop_ids(alice.id) {
        let user = graph.get_user(id).ok_or("Referenced user is missing")?;
        println!("  {}", user.name);
    }

    Ok(())
}
