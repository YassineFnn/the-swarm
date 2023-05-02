use serde::{Deserialize, Serialize};

use crate::consensus::graph::SyncJobs;

/// Identifier for whole data unit (not split into shards). For example,
/// this can be ID of vector. Shards of the vector will have the same
/// identifier
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct Vid(pub u64);

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Hash)]
pub struct Sid(pub u64);

/// Type/struct that represents unit of data stored on nodes.
/// Should be actual data shard (erasure coded) in the future, but
/// right now for demonstration purposes, represents vector(array) of size 4.
// #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub type Shard = [i32; 4];

/// for now just splits into three parts
pub type Data = [i32; 12];

/// Graph representation that is passed on random gossip.
pub type Graph = SyncJobs<Vid, Sid>;
