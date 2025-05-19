use crate::topic::{QoS, TopicFilter, TopicName};
use rand::random;
use std::collections::HashMap;
use std::ops::DerefMut;

/// ClientId is the internal id assigned to the client by the server, u64 will never overflow, so we
/// can safely assume this is unique
pub type ClientId = u64;

/// The TopicTree is a tree structure containing all the routing information for the subscribers
/// Subscriptions are added or removed from this structure and all clients that are subscribed to a
/// topic can be queried from here
#[derive(Default, Debug, Clone)]
pub struct TopicTree {
    root_node: TopicNode,
    subscribers: u64,
}

impl TopicTree {
    pub fn get_routes(&self, publish_topic: &TopicName) -> Vec<SubScriber> {
        let mut results = Vec::with_capacity(self.subscribers as usize);
        // self.root_node.get_routes_rec(0, publish_topic, &mut results);
        // self.root_node.get_routes(publish_topic, &mut results);
        self.root_node.get_routes_arr(publish_topic, &mut results);
        results
    }

    pub fn add_subscription(
        &mut self,
        topic_filter: TopicFilter,
        client_id: ClientId,
        qos: QoS,
    ) {
        self.root_node.add_subscriber(topic_filter, client_id, qos);
        self.subscribers += 1;
    }
}

/// The TopicNode is the core of the TopicTree structure, the single level wildcard and multilevel
/// wildcards are seperate fields in the struct to avoid additional hashmap lookups.
#[derive(Default, Debug, Clone)]
struct TopicNode {
    multi_level_wildcard: Option<Box<RouteInfo>>,
    single_level_wildcard: Option<Box<TopicNode>>,
    sub_nodes: HashMap<String, TopicNode>,
    content: RouteInfo,
}

impl TopicNode {
    /// All 3 implementations of get routes do the same thing, the array version is currently the
    /// fastest and therefore used, but it is also the least readable
    fn get_routes(&self, publish_topic: &TopicName, results: &mut Vec<SubScriber>) {
        let mut vec1: Vec<&TopicNode> = Vec::with_capacity(3);
        let mut vec2: Vec<&TopicNode> = Vec::with_capacity(3);
        vec2.push(self);
        for i in 0..publish_topic.length {
            // let topiclevel = &publish_topic.topic_levels[i];
            let topiclevel = publish_topic.get_part(i).unwrap();
            std::mem::swap(&mut vec1, &mut vec2);
            vec2.clear();
            for curr_node in vec1.iter() {
                if let Some(routeinfo) = curr_node.multi_level_wildcard.as_deref() {
                    results.extend(routeinfo.get_subs())
                }
                if let Some(single_wildcard_match) = curr_node.single_level_wildcard.as_deref() {
                    vec2.push(single_wildcard_match);
                }
                if let Some(literal_match) = curr_node.sub_nodes.get(topiclevel) {
                    vec2.push(literal_match);
                }
            }
        }
        for final_node in vec2 {
            let literal_match = final_node.content.get_subs();
            results.extend(literal_match);
        }
    }

    fn get_routes_arr(&self, publish_topic: &TopicName, results: &mut Vec<SubScriber>) {
        let mut curr_iter: bool = false;
        let mut iter_len = [0usize, 1usize];
        let mut iter_arr = [[self, self], [self, self]];
        for i in 0..publish_topic.length {
            // let topiclevel = &publish_topic.topic_levels[i];
            let topiclevel = publish_topic.get_part(i).unwrap();
            iter_len[curr_iter as usize] = 0;
            curr_iter = !curr_iter;
            for j in 0..iter_len[curr_iter as usize] {
                let curr_node = iter_arr[curr_iter as usize][j];
                if let Some(routeinfo) = curr_node.multi_level_wildcard.as_deref() {
                    results.extend(routeinfo.get_subs())
                }
                if let Some(single_wildcard_match) = curr_node.single_level_wildcard.as_deref() {
                    iter_arr[!curr_iter as usize][iter_len[!curr_iter as usize]] =
                        single_wildcard_match;
                    iter_len[!curr_iter as usize] += 1;
                }
                if let Some(literal_match) = curr_node.sub_nodes.get(topiclevel) {
                    iter_arr[!curr_iter as usize][iter_len[!curr_iter as usize]] = literal_match;
                    iter_len[!curr_iter as usize] += 1;
                }
            }
        }
        curr_iter = !curr_iter;
        for j in 0..iter_len[curr_iter as usize] {
            let final_node = iter_arr[curr_iter as usize][j];
            let literal_match = final_node.content.get_subs();
            results.extend(literal_match);
        }
    }

    fn get_routes_rec(
        &self,
        curr_level: usize,
        publish_topic: &TopicName,
        results: &mut Vec<SubScriber>,
    ) {
        if let Some(routeinfo) = self.multi_level_wildcard.as_deref() {
            results.extend(routeinfo.get_subs())
        }
        if curr_level < publish_topic.length {
            let topiclevel = publish_topic.get_part(curr_level).unwrap();
            if let Some(single_wildcard_match) = self.single_level_wildcard.as_deref() {
                single_wildcard_match.get_routes_rec(curr_level + 1, publish_topic, results)
            };
            if let Some(literal_match) = self.sub_nodes.get(topiclevel) {
                literal_match.get_routes_rec(curr_level + 1, publish_topic, results);
            }
        } else {
            results.extend(self.content.get_subs())
        }
    }

    fn add_subscriber(&mut self, topic_filter: TopicFilter, client_id: ClientId, qos: QoS) {
        let mut curr_node = self;
        let subscriber = SubScriber { client_id, qos };
        for topiclevel in &topic_filter.topic_levels {
            match topiclevel.as_str() {
                "+" => {
                    curr_node = curr_node.get_single_level_wildcard_node_or_create();
                }
                "#" => {
                    curr_node.add_multi_level_wildcard_if_not_exists();
                    let routeinfo = curr_node.multi_level_wildcard.as_deref_mut().unwrap();
                    match topic_filter.shared_group_name.clone() {
                        None => {
                            routeinfo.add_client_subscription(subscriber);
                        }
                        Some(shared_group) => {
                            routeinfo.add_shared_subscription(subscriber, shared_group)
                        }
                    }
                    return;
                }
                _ => {
                    curr_node = curr_node.get_sub_node_or_create(&topiclevel);
                }
            }
        }

        match topic_filter.shared_group_name {
            None => {
                curr_node.content.add_client_subscription(subscriber);
            }
            Some(shared_group) => curr_node
                .content
                .add_shared_subscription(subscriber, shared_group),
        }
    }

    fn get_sub_node_or_create(&mut self, topic_level: &String) -> &mut Self {
        if !self.sub_nodes.contains_key(topic_level.clone().as_str()) {
            self.sub_nodes
                .insert(topic_level.clone(), TopicNode::default());
        }
        self.sub_nodes.get_mut(topic_level).unwrap()
    }

    fn get_single_level_wildcard_node_or_create(&mut self) -> &mut Self {
        if self.single_level_wildcard.is_none() {
            self.single_level_wildcard = Some(Box::new(TopicNode::default()));
        }
        self.single_level_wildcard.as_mut().unwrap().deref_mut()
    }

    fn add_multi_level_wildcard_if_not_exists(&mut self) {
        if self.multi_level_wildcard.is_none() {
            self.multi_level_wildcard = Some(Box::new(RouteInfo::default()));
        }
    }
}

/// The RouteInfo contains all the info about the subscriptions
#[derive(Default, Debug, Clone)]
struct RouteInfo {
    client_subscriptions: Vec<SubScriber>,
    shared_subscriptions: Vec<ClientGroup>,
}

impl RouteInfo {
    fn get_subs(&self) -> Vec<SubScriber> {
        let mut subs = self.client_subscriptions.clone();
        let shared_sub: Vec<SubScriber> = self
            .shared_subscriptions
            .iter()
            .map(|x| x.get_next_client())
            .collect();
        subs.extend(shared_sub);
        subs
    }

    fn add_client_subscription(&mut self, subscriber: SubScriber) {
        self.client_subscriptions.push(subscriber)
    }

    fn add_shared_subscription(&mut self, subscriber: SubScriber, shared_group: String) {
        if let Some(group) = self
            .shared_subscriptions
            .iter_mut()
            .find(|x| x.group_id == shared_group)
        {
            group.clients.push(subscriber)
        } else {
            self.shared_subscriptions
                .push(ClientGroup::new(shared_group, subscriber))
        }
    }
}

/// The ClientGroup represents a single shared subscription. As the
#[derive(Debug, Clone)]
struct ClientGroup {
    group_id: String,
    clients: Vec<SubScriber>,
    num_clients: usize,
}

impl ClientGroup {
    fn new(group_id: String, subscriber: SubScriber) -> Self {
        Self {
            group_id,
            clients: Vec::from([subscriber]),
            num_clients: 1,
        }
    }
    fn get_client_by_number(&self, id: u64) -> SubScriber {
        let idx = id as usize % self.num_clients;
        self.clients[idx].clone()
    }

    fn get_next_client(&self) -> SubScriber {
        self.get_client_by_number(random())
    }

    fn add_subscriber(&mut self, subscriber: SubScriber) {
        self.clients.push(subscriber);
    }
}

#[derive(Clone, Debug)]
pub struct SubScriber {
    pub client_id: ClientId,
    pub qos: QoS,
}
