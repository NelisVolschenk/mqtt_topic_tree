use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

pub type QoS = u8;

/// ClientId is the internal id assigned to the client by the server, u64 will never overflow, so we
/// can safely assume this is unique
pub type ClientId = u64;

/// The TopicTree is a tree structure containing all the routing information for the subscribers
/// Subscriptions are added or removed from this structure and all clients that are subscribed to a
/// topic can be queried from here
struct TopicTree {
    nodes: HashMap<String, TopicNode>
}

struct TopicNode {
    content: RouteInfo,
    sub_nodes: HashMap<String, TopicNode>
}

/// The RouteInfo contains all the info about the subscriptions
struct RouteInfo {
    client_subscriptions: Vec<(ClientId, QoS)>,
    shared_subscriptions: HashMap<String, ClientGroup>
}

/// The ClientGroup represents a single shared subscription. The curr_location counter is an
/// AtomicU64 in and Arc to ensure continuity between the two copies of the TopicTree (I might
/// change this to just a normal u64 and sacrifice that continuity in favour of speed as the atomic
/// access will invalidate cache lines
struct ClientGroup {
    curr_location: Arc<AtomicU64>,
    clients: Vec<(ClientId, QoS)>
}

impl TopicTree {


}

impl Default for TopicTree {
    fn default() -> Self {
        Self {
            nodes: HashMap::default()
        }
    }
}

