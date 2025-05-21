use crate::topic::{TopicFilter, TopicName};
use crate::{ClientId, QoS};
use rand::random;
use std::collections::HashMap;
use std::ops::DerefMut;

/// The TopicTree is a tree structure containing all the routing information for the subscribers
/// Subscriptions are added or removed from this structure and all clients that are subscribed to a
/// topic can be queried from here
#[derive(Default, Debug, Clone)]
pub struct TopicTree {
    root_node: TopicNode,
    subscribers: u64,
}

impl TopicTree {
    pub fn get_subscriptions(&self, publish_topic: &TopicName) -> Vec<SubScriber> {
        let mut results = Vec::with_capacity(self.subscribers as usize);
        // self.root_node
        //     .get_subscriptions_rec(0, publish_topic, &mut results);
        // self.root_node
        //     .get_subscriptions(publish_topic, &mut results);
        self.root_node
            .get_subscriptions_arr(publish_topic, &mut results);
        results
    }

    pub fn add_subscription(&mut self, topic_filter: TopicFilter, client_id: ClientId, qos: QoS) {
        self.root_node.add_subscriber(topic_filter, client_id, qos);
        self.subscribers += 1;
    }

    pub fn remove_subscription(
        &mut self,
        topic_filter: TopicFilter,
        client_id: ClientId,
    ) {
        self.root_node.remove_subscriber(topic_filter, client_id);
        self.subscribers -= 1;
    }
}

/// The TopicNode is the core of the TopicTree structure, the single level wildcard and multilevel
/// wildcards are seperate fields in the struct to avoid additional hashmap lookups.
#[derive(Default, Debug, Clone)]
struct TopicNode {
    multi_level_wildcard: Option<Box<SubscriptionInfo>>,
    single_level_wildcard: Option<Box<TopicNode>>,
    sub_nodes: HashMap<String, TopicNode>,
    content: SubscriptionInfo,
}

impl TopicNode {
    /// All 3 implementations of get routes do the same thing, the array version is currently the
    /// fastest and therefore used, but it is also the least readable
    #[allow(dead_code)]
    fn get_subscriptions(&self, publish_topic: &TopicName, results: &mut Vec<SubScriber>) {
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
                    results.extend(routeinfo.get_subscriptions())
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
            let literal_match = final_node.content.get_subscriptions();
            results.extend(literal_match);
        }
    }

    #[allow(dead_code)]
    fn get_subscriptions_arr(&self, publish_topic: &TopicName, results: &mut Vec<SubScriber>) {
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
                    results.extend(routeinfo.get_subscriptions())
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
            let literal_match = final_node.content.get_subscriptions();
            results.extend(literal_match);
        }
    }

    #[allow(dead_code)]
    fn get_subscriptions_rec(
        &self,
        curr_level: usize,
        publish_topic: &TopicName,
        results: &mut Vec<SubScriber>,
    ) {
        if let Some(routeinfo) = self.multi_level_wildcard.as_deref() {
            results.extend(routeinfo.get_subscriptions())
        }
        if curr_level < publish_topic.length {
            let topiclevel = publish_topic.get_part(curr_level).unwrap();
            if let Some(single_wildcard_match) = self.single_level_wildcard.as_deref() {
                single_wildcard_match.get_subscriptions_rec(curr_level + 1, publish_topic, results)
            };
            if let Some(literal_match) = self.sub_nodes.get(topiclevel) {
                literal_match.get_subscriptions_rec(curr_level + 1, publish_topic, results);
            }
        } else {
            results.extend(self.content.get_subscriptions())
        }
    }

    fn add_subscriber(&mut self, topic_filter: TopicFilter, client_id: ClientId, qos: QoS) {
        let mut curr_node = self;
        for i in 0..topic_filter.length {
            let topic_level = topic_filter.get_part(i).unwrap();
            match topic_level {
                "+" => {
                    curr_node = curr_node.get_single_level_wildcard_node_or_create();
                }
                "#" => {
                    curr_node.add_multi_level_wildcard_if_not_exists();
                    let routeinfo = curr_node.multi_level_wildcard.as_deref_mut().unwrap();
                    match topic_filter.shared_group_name.clone() {
                        None => {
                            routeinfo.add_client_subscription(client_id, qos);
                        }
                        Some(shared_group) => {
                            routeinfo.add_shared_subscription(client_id, qos, shared_group)
                        }
                    }
                    return;
                }
                _ => {
                    curr_node = curr_node.get_sub_node_or_create(topic_level);
                }
            }
        }

        match topic_filter.shared_group_name {
            None => {
                curr_node.content.add_client_subscription(client_id, qos);
            }
            Some(shared_group) => curr_node
                .content
                .add_shared_subscription(client_id, qos, shared_group),
        }
    }

    fn remove_subscriber(
        &mut self,
        topic_filter: TopicFilter,
        client_id: ClientId,
    ) {
        let mut curr_node = self;
        let sub_info: &mut SubscriptionInfo;
        let topic_level: &str = "";
        for i in 0..topic_filter.length {
            let topic_level = topic_filter.get_part(i).unwrap();
            match topic_level {
                "+" => {
                    if curr_node.single_level_wildcard.is_none() {
                        return;
                    } else {
                        curr_node = curr_node.single_level_wildcard.as_mut().unwrap().deref_mut()
                    }
                }
                "#" => {
                    if i != topic_filter.length - 1 {return;}
                    if curr_node.multi_level_wildcard.is_none() {
                        return;
                    }
                }
                _ => {
                    if !curr_node.sub_nodes.contains_key(topic_level) {
                        return;
                    } else {
                        curr_node = curr_node.sub_nodes.get_mut(topic_level).unwrap();
                    }
                }
            }
        }
        match topic_level {
            "#" => {sub_info = curr_node.multi_level_wildcard.as_mut().unwrap().deref_mut();}
            _ => {sub_info = &mut curr_node.content}
        }

        match topic_filter.shared_group_name {
            None => {
                sub_info.remove_client_subscription(client_id)
            }
            Some(shared_group) =>  {
                sub_info.remove_shared_subscription(client_id, shared_group)
            }
        }
    }

    fn get_sub_node_or_create(&mut self, topic_level: &str) -> &mut Self {
        if !self.sub_nodes.contains_key(topic_level) {
            self.sub_nodes
                .insert(topic_level.to_owned(), TopicNode::default());
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
            self.multi_level_wildcard = Some(Box::new(SubscriptionInfo::default()));
        }
    }
}

/// The RouteInfo contains all the info about the subscriptions
#[derive(Default, Debug, Clone)]
struct SubscriptionInfo {
    client_subscriptions: HashMap<ClientId, QoS>,
    shared_subscriptions: Vec<ClientGroup>,
}

impl SubscriptionInfo {
    fn get_subscriptions(&self) -> Vec<SubScriber> {
        let mut subs: Vec<SubScriber> = self.client_subscriptions
            .iter()
            .map(|x| SubScriber{client_id: x.0.clone(), qos: x.1.clone() })
            .collect();
        let shared_sub: Vec<SubScriber> = self
            .shared_subscriptions
            .iter()
            .map(|x| x.get_next_client())
            .collect();
        subs.extend(shared_sub);
        subs
    }

    fn add_client_subscription(&mut self, client_id: ClientId, qos: QoS) {
        self.client_subscriptions.insert(client_id, qos);
    }

    fn remove_client_subscription(&mut self, client_id: ClientId) {
        let _res = self.client_subscriptions.remove(&client_id);
    }

    fn add_shared_subscription(&mut self, client_id: ClientId, qos: QoS, shared_group: String) {
        let subscriber = SubScriber{client_id, qos};
        if let Some(group) = self
            .shared_subscriptions
            .iter_mut()
            .find(|x| x.group_id == shared_group)
        {
            group.add_subscriber(subscriber);
        } else {
            self.shared_subscriptions
                .push(ClientGroup::new(shared_group, subscriber))
        }
    }

    fn remove_shared_subscription(&mut self, client_id: ClientId, shared_group: String) {
        if let Some(group) = self
            .shared_subscriptions
            .iter_mut()
            .find(|x| x.group_id == shared_group) {
            group.remove_subscriber(client_id)
        }
    }
}

/// The ClientGroup represents a single shared subscription.
#[derive(Debug, Clone)]
struct ClientGroup {
    group_id: String,
    clients: Vec<SubScriber>,
}

impl ClientGroup {
    fn new(group_id: String, subscriber: SubScriber) -> Self {
        Self {
            group_id,
            clients: Vec::from([subscriber]),
        }
    }
    fn get_client_by_number(&self, id: u64) -> SubScriber {
        let idx = id as usize % self.clients.len();
        self.clients[idx].clone()
    }

    fn get_next_client(&self) -> SubScriber {
        self.get_client_by_number(random())
    }

    fn add_subscriber(&mut self, subscriber: SubScriber) {
        self.clients.push(subscriber);
    }

    fn remove_subscriber(&mut self, client_id: ClientId) {
        if let Some(idx) = self.clients
            .iter()
            .position(|x| x.client_id == client_id) {
            self.clients.remove(idx);
        }

    }
}

#[derive(Clone, Debug)]
pub struct SubScriber {
    pub client_id: ClientId,
    pub qos: QoS,
}
