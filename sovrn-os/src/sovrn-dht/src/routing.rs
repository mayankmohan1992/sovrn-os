
//! Kademlia-style routing table for DHT

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingEntry {
    pub node_id: String,
    pub public_key: String,
    pub addr: String,
    pub distance: usize,
    pub last_seen: i64,
}

pub struct RoutingTable {
    local_id: String,
    k: usize,
    buckets: BTreeMap<u8, Vec<RoutingEntry>>,
}

impl RoutingTable {
    pub fn new(local_id: String, k: usize) -> Self {
        Self {
            local_id,
            k,
            buckets: BTreeMap::new(),
        }
    }

    /// Calculate XOR distance between two node IDs (hex strings)
    fn distance(a: &str, b: &str) -> u8 {
        let a_bytes = hex::decode(a).unwrap_or_default();
        let b_bytes = hex::decode(b).unwrap_or_default();
        let mut dist = 0u8;
        for (ab, bb) in a_bytes.iter().zip(b_bytes.iter()) {
            dist = dist.wrapping_add(ab ^ bb);
        }
        dist
    }

    fn bucket_index(&self, distance: u8) -> u8 {
        // 256 buckets based on leading zero bits of distance
        // Simplified: use distance directly as bucket index
        distance
    }

    pub fn insert(&mut self, entry: RoutingEntry) {
        let distance = Self::distance(&self.local_id, &entry.node_id);
        let bucket_idx = self.bucket_index(distance.min(255));
        let bucket = self.buckets.entry(bucket_idx).or_insert_with(Vec::new);

        // Check if entry already exists
        if let Some(pos) = bucket.iter().position(|e| e.node_id == entry.node_id) {
            bucket[pos] = entry;
        } else if bucket.len() < self.k {
            bucket.push(entry);
        } else {
            // Bucket full, drop the entry (Kademlia: ping oldest to evict)
        }
    }

    pub fn get_closest_peers(&self, target: &str, count: usize) -> Vec<RoutingEntry> {
        let distance = Self::distance(&self.local_id, target);
        let bucket_idx = self.bucket_index(distance.min(255));

        let mut peers: Vec<RoutingEntry> = Vec::new();

        // Start from closest bucket
        if let Some(bucket) = self.buckets.get(&bucket_idx) {
            peers.extend(bucket.iter().cloned());
        }

        // Expand to neighboring buckets
        for offset in 1..=3 {
            if let Some(bucket) = self.buckets.get(&bucket_idx.wrapping_add(offset)) {
                peers.extend(bucket.iter().cloned());
            }
            if let Some(bucket) = self.buckets.get(&bucket_idx.wrapping_sub(offset)) {
                peers.extend(bucket.iter().cloned());
            }
        }

        peers.sort_by_key(|p| Self::distance(&p.node_id, target));
        peers.truncate(count);
        peers
    }

    pub fn bucket_count(&self) -> usize {
        self.buckets.values().map(|b| b.len()).sum()
    }

    pub fn remove(&mut self, node_id: &str) {
        for bucket in self.buckets.values_mut() {
            bucket.retain(|e| e.node_id != node_id);
        }
    }
}
