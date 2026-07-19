use graph_shard_lab::sharded::{Placement, ShardedGraph};

fn build_tiny_graph(placement: Placement) -> ShardedGraph {
    let mut graph = ShardedGraph::with_placement(2, placement).unwrap();

    for id in 1..=8 {
        graph.add_user(id, &format!("user-{id}")).unwrap();
    }

    let edges = [
        (1, 2),
        (1, 3),
        (2, 4),
        (3, 4),
        (5, 6),
        (5, 7),
        (6, 8),
        (7, 8),
        (4, 5),
        (8, 1),
    ];

    for (source, target) in edges {
        graph.add_follow(source, target).unwrap();
    }

    graph
}

#[test]
fn tiny_graph_explains_hash_vs_community_placement() {
    let hash_graph = build_tiny_graph(Placement::Hash);

    let community_graph = build_tiny_graph(Placement::Community { community_size: 4 });

    let hash_result = hash_graph.get_two_hop_with_stats(1);
    let community_result = community_graph.get_two_hop_with_stats(1);

    assert_eq!(hash_result.user_ids, vec![4]);
    assert_eq!(community_result.user_ids, vec![4]);

    assert_eq!(hash_result.shards_touched, 2);
    assert_eq!(hash_result.cross_shard_hops, 2);

    assert_eq!(community_result.shards_touched, 1);
    assert_eq!(community_result.cross_shard_hops, 0);
}
