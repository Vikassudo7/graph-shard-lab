use crate::error::{GraphError, Result};
use crate::sharded::{Placement, ShardedGraph};

#[derive(Debug, Clone)]
pub struct SplitCommunity {
    pub community_id: usize,
    pub original_size: u64,
    pub split_into: Vec<SplitChunk>,
}

#[derive(Debug, Clone)]
pub struct SplitChunk {
    pub shard_id: usize,
    pub user_range: (u64, u64),
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct SplittingPlan {
    pub communities: Vec<SplitCommunity>,
    pub total_shards: usize,
}

pub fn plan_community_splitting(
    community_sizes: &[u64],
    shard_count: usize,
    max_community_size: u64,
) -> Result<SplittingPlan> {
    if shard_count == 0 {
        return Err(GraphError::ZeroShardCount);
    }

    if community_sizes.is_empty() {
        return Err(GraphError::EmptyCommunities);
    }

    if community_sizes.contains(&0) {
        return Err(GraphError::ZeroCommunitySizes);
    }

    let mut communities = Vec::new();

    for (community_id, &size) in community_sizes.iter().enumerate() {
        if size <= max_community_size {
            communities.push(SplitCommunity {
                community_id,
                original_size: size,
                split_into: vec![SplitChunk {
                    shard_id: community_id % shard_count,
                    user_range: (0, size),
                    size,
                }],
            });
        } else {
            let chunks = compute_split_chunks(size, shard_count, max_community_size);

            communities.push(SplitCommunity {
                community_id,
                original_size: size,
                split_into: chunks,
            });
        }
    }

    Ok(SplittingPlan {
        communities,
        total_shards: shard_count,
    })
}

fn compute_split_chunks(size: u64, shard_count: usize, max_size: u64) -> Vec<SplitChunk> {
    let mut chunks = Vec::new();
    let chunk_size = size.min(max_size);
    let num_chunks = size.div_ceil(chunk_size);

    let mut offset = 0u64;

    for chunk_index in 0..num_chunks {
        let remaining = size - offset;
        let this_chunk_size = remaining.min(chunk_size);

        chunks.push(SplitChunk {
            shard_id: (chunk_index as usize) % shard_count,
            user_range: (offset, offset + this_chunk_size),
            size: this_chunk_size,
        });

        offset += this_chunk_size;
    }

    chunks
}

pub fn apply_splitting_plan(shard_count: usize, plan: &SplittingPlan) -> Result<ShardedGraph> {
    let mut placement_map = Vec::new();
    let mut community_sizes = Vec::new();

    for community in &plan.communities {
        community_sizes.push(community.original_size);

        let total_chunk_size: u64 = community.split_into.iter().map(|c| c.size).sum();

        let primary_shard = community
            .split_into
            .first()
            .map(|c| c.shard_id)
            .unwrap_or(0);

        placement_map.push(primary_shard);

        if community.split_into.len() > 1 {
            let _ = total_chunk_size;
        }
    }

    let mut graph = ShardedGraph::with_placement(
        shard_count,
        Placement::BalancedCommunity {
            community_sizes,
            community_to_shard: placement_map,
        },
    )?;

    for community in &plan.communities {
        let base_user_id: u64 = plan.communities[..community.community_id]
            .iter()
            .map(|c| c.original_size)
            .sum::<u64>()
            + 1;

        for chunk in &community.split_into {
            for user_offset in 0..chunk.size {
                let user_id = base_user_id + chunk.user_range.0 + user_offset;
                let _ = graph.add_user(user_id, &format!("user-{user_id}"));
            }
        }
    }

    Ok(graph)
}

pub fn create_split_placement(
    community_sizes: &[u64],
    shard_count: usize,
    max_community_size: u64,
) -> Result<Placement> {
    let plan = plan_community_splitting(community_sizes, shard_count, max_community_size)?;

    let community_to_shard: Vec<usize> = plan
        .communities
        .iter()
        .map(|c| {
            c.split_into
                .first()
                .map(|chunk| chunk.shard_id)
                .unwrap_or(0)
        })
        .collect();

    Ok(Placement::BalancedCommunity {
        community_sizes: community_sizes.to_vec(),
        community_to_shard,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_communities_are_not_split() {
        let plan = plan_community_splitting(&[100, 200, 300], 4, 500).unwrap();

        for community in &plan.communities {
            assert_eq!(community.split_into.len(), 1);
        }
    }

    #[test]
    fn large_community_is_split() {
        let plan = plan_community_splitting(&[1000], 4, 400).unwrap();

        assert_eq!(plan.communities.len(), 1);
        assert!(plan.communities[0].split_into.len() > 1);
    }

    #[test]
    fn split_chunks_cover_original_size() {
        let plan = plan_community_splitting(&[1000], 4, 400).unwrap();

        let total: u64 = plan.communities[0].split_into.iter().map(|c| c.size).sum();

        assert_eq!(total, 1000);
    }

    #[test]
    fn split_chunks_do_not_exceed_max_size() {
        let plan = plan_community_splitting(&[1000], 4, 400).unwrap();

        for community in &plan.communities {
            for chunk in &community.split_into {
                assert!(chunk.size <= 400);
            }
        }
    }

    #[test]
    fn creates_valid_sharded_graph() {
        let plan = plan_community_splitting(&[100, 200, 300], 4, 500).unwrap();

        let graph = apply_splitting_plan(4, &plan).unwrap();

        assert_eq!(graph.shard_count(), 4);
        assert!(graph.user_count() > 0);
    }

    #[test]
    fn create_split_placement_returns_valid_placement() {
        let placement = create_split_placement(&[100, 200, 300], 4, 500).unwrap();

        if let Placement::BalancedCommunity {
            community_sizes,
            community_to_shard,
        } = placement
        {
            assert_eq!(community_sizes.len(), 3);
            assert_eq!(community_to_shard.len(), 3);
        } else {
            panic!("Expected BalancedCommunity placement");
        }
    }

    #[test]
    fn rejects_zero_shard_count() {
        assert!(plan_community_splitting(&[100, 200], 0, 500).is_err());
    }

    #[test]
    fn rejects_empty_communities() {
        assert!(plan_community_splitting(&[], 4, 500).is_err());
    }

    #[test]
    fn rejects_zero_community_sizes() {
        assert!(plan_community_splitting(&[100, 0, 200], 4, 500).is_err());
    }
}
