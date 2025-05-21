use std::sync::{Arc};
use left_right::{Absorb, ReadHandle, WriteHandle};
use parking_lot::Mutex;
use crate::client_types::{ClientId, QoS};
use crate::sync::TopicTreeOperations::AddSubscription;
use crate::topic::{TopicFilter, TopicName};
use crate::topic_tree::{SubScriber, TopicTree};

pub enum  TopicTreeOperations {
    AddSubscription(TopicFilter, ClientId, QoS),
}

impl Absorb<TopicTreeOperations> for TopicTree {
    fn absorb_first(&mut self, operation: &mut TopicTreeOperations, _: &Self) {
        match operation {
            AddSubscription(TopicFilter, ClientId, QoS) => {
                self.add_subscription(TopicFilter.clone(), ClientId.clone(), QoS.clone())
            }
        }
    }

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone();
    }
}

pub struct MqttTopicTree {
    read_handle: ReadHandle<TopicTree>,
    write_handle: Arc<Mutex<WriteHandle<TopicTree, TopicTreeOperations>>>
}

impl MqttTopicTree {
    pub fn new() -> Self {
        let (write, read) = left_right::new::<TopicTree, TopicTreeOperations>();
        Self {
            read_handle: read,
            write_handle: Arc::new(Mutex::new(write))
        }
    }

    pub fn add_subscription(
        &self,
        topic_filter: TopicFilter,
        client_id: ClientId,
        qos: QoS,
    ) {
        let mut write_handle = self.write_handle.lock();
        let operation = AddSubscription(topic_filter, client_id, qos);
        write_handle.append(operation);
        write_handle.publish();
    }

    pub fn get_subscriptions(&self, publish_topic: &TopicName) -> Vec<SubScriber> {
        let a = self.read_handle.enter().unwrap();
        a.get_subscriptions(publish_topic)
    }
}
