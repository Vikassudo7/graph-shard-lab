use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    DuplicateUser(u64),
    UserNotFound(u64),
    SourceUserNotFound(u64),
    TargetUserNotFound(u64),
    ShardNotFound(u64),
    OutsideCommunityRange(u64),
    ZeroShardCount,
    ZeroCacheCapacity,
    ZeroByteCapacity,
    ZeroCommunitySize,
    EmptyCommunities,
    ZeroCommunitySizes,
    CommunitySizeMismatch,
    InvalidShardInAssignment,
    CommunitySizeOverflow,
    CachingDisabled,
    WarmEntryLimitZero,
    InvalidWorkerConfig,
    WorkerStopped(usize),
    WorkerDropped(usize, &'static str),
    CacheAccountingMismatch,
    CorrectnessMismatch(u64),
    StartupWindowMismatch,
    LatencySampleCountOverflow,
    EmptyLatencySamples,
    ZeroRepetitions,
    IoError(String),
    ZeroUserCount,
    ZeroEdgeCount,
    ZeroHubCount,
    HubCountTooLarge,
    EdgesPerUserTooLarge,
    HubEdgesExceedTotal,
    TooManyHubEdges,
    TooManyRegularEdges,
    UserCountNotDivisible,
    LocalEdgesExceedTotal,
    LocalEdgesExceedCommunitySize,
    CrossEdgesExceedExternal,
    CommunityTargetCountMismatch(String),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateUser(id) => write!(f, "User {id} already exists"),
            Self::UserNotFound(id) => write!(f, "User {id} does not exist"),
            Self::SourceUserNotFound(id) => write!(f, "Source user {id} does not exist"),
            Self::TargetUserNotFound(id) => write!(f, "Target user {id} does not exist"),
            Self::ShardNotFound(id) => write!(f, "Cannot find shard for user {id}"),
            Self::OutsideCommunityRange(id) => {
                write!(f, "User {id} is outside the configured community ranges")
            }
            Self::ZeroShardCount => write!(f, "Shard count must be greater than zero"),
            Self::ZeroCacheCapacity => write!(f, "Cache capacity must be greater than zero"),
            Self::ZeroByteCapacity => {
                write!(f, "Cache byte capacity must be greater than zero")
            }
            Self::ZeroCommunitySize => {
                write!(f, "Community size must be greater than zero")
            }
            Self::EmptyCommunities => write!(f, "At least one community is required"),
            Self::ZeroCommunitySizes => {
                write!(f, "Community sizes must be greater than zero")
            }
            Self::CommunitySizeMismatch => {
                write!(f, "Every community must have a shard assignment")
            }
            Self::InvalidShardInAssignment => {
                write!(f, "Community assignment contains an invalid shard")
            }
            Self::CommunitySizeOverflow => {
                write!(f, "Total community size is too large")
            }
            Self::CachingDisabled => write!(f, "Caching is disabled for this ShardedGraph"),
            Self::WarmEntryLimitZero => {
                write!(f, "Warm entry limit must be greater than zero")
            }
            Self::InvalidWorkerConfig => {
                write!(f, "Channel capacity must be greater than zero")
            }
            Self::WorkerStopped(id) => write!(f, "Shard worker {id} has stopped"),
            Self::WorkerDropped(id, msg) => {
                write!(f, "Shard worker {id} dropped the {msg} response")
            }
            Self::CacheAccountingMismatch => {
                write!(f, "Cache accounting does not match workload")
            }
            Self::CorrectnessMismatch(id) => {
                write!(f, "Correctness mismatch for source user {id}")
            }
            Self::StartupWindowMismatch => {
                write!(f, "Startup window must divide evenly by edges per user")
            }
            Self::LatencySampleCountOverflow => {
                write!(f, "Latency sample count is too large")
            }
            Self::EmptyLatencySamples => {
                write!(f, "At least one latency sample is required")
            }
            Self::ZeroRepetitions => {
                write!(f, "Benchmark repetitions must be greater than zero")
            }
            Self::IoError(msg) => write!(f, "{msg}"),
            Self::ZeroUserCount => write!(f, "User count must be greater than zero"),
            Self::ZeroEdgeCount => write!(f, "Edges per user must be greater than zero"),
            Self::ZeroHubCount => write!(f, "Hub count must be greater than zero"),
            Self::HubCountTooLarge => write!(f, "Hub count must be smaller than user count"),
            Self::EdgesPerUserTooLarge => {
                write!(f, "Edges per user must be smaller than user count")
            }
            Self::HubEdgesExceedTotal => {
                write!(f, "Hub edges cannot exceed total edges")
            }
            Self::TooManyHubEdges => write!(f, "Too many hub edges requested"),
            Self::TooManyRegularEdges => write!(f, "Too many regular edges requested"),
            Self::UserCountNotDivisible => {
                write!(f, "User count must divide evenly into communities")
            }
            Self::LocalEdgesExceedTotal => {
                write!(f, "Local edges cannot exceed total edges")
            }
            Self::LocalEdgesExceedCommunitySize => {
                write!(f, "Too many local edges for the community size")
            }
            Self::CrossEdgesExceedExternal => {
                write!(f, "Too many cross-community edges requested")
            }
            Self::CommunityTargetCountMismatch(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GraphError {}

pub type Result<T> = std::result::Result<T, GraphError>;
