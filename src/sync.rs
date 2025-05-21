use std::sync::{Arc};
use left_right::{Absorb, ReadHandle, ReadHandleFactory, WriteHandle};
use parking_lot::Mutex;
use crate::sync::TopicTreeOperations::{AddSubscription, RemoveSubscription};
use crate::{ClientId, QoS, Subscriber, TopicFilter, TopicName, TopicTree};

pub enum  TopicTreeOperations {
    AddSubscription(TopicFilter, ClientId, QoS),
    RemoveSubscription(TopicFilter, ClientId),
}

impl Absorb<TopicTreeOperations> for TopicTree {
    fn absorb_first(&mut self, operation: &mut TopicTreeOperations, _: &Self) {
        match operation {
            AddSubscription(topic_filter, client_id, qos) => {
                self.add_subscription(topic_filter.clone(), client_id.clone(), qos.clone())
            }
            RemoveSubscription(topic_filer, client_id) => {
                self.remove_subscription(topic_filer.clone(), client_id.clone())
            }
        }
    }

    fn sync_with(&mut self, first: &Self) {
        *self = first.clone();
    }
}

pub struct MqttTopicTreeCreator {
    write_handle: Arc<Mutex<WriteHandle<TopicTree, TopicTreeOperations>>>,
    factory: ReadHandleFactory<TopicTree>
}

impl MqttTopicTreeCreator {
    pub fn to_mqtt_topic_tree(self) -> MqttTopicTree {
        let read_handle = self.factory.handle();
        MqttTopicTree {
            read_handle,
            write_handle: self.write_handle
        }
    }
}

impl Default for MqttTopicTreeCreator {
    fn default() -> Self {
        let (write, read) = left_right::new::<TopicTree, TopicTreeOperations>();
        let factory = write.factory();
        Self {
            write_handle: Arc::new(Mutex::new(write)),
            factory
        }
    }
}

#[derive(Clone)]
pub struct MqttTopicTree {
    read_handle: ReadHandle<TopicTree>,
    write_handle: Arc<Mutex<WriteHandle<TopicTree, TopicTreeOperations>>>
}

impl MqttTopicTree {

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

    pub fn remove_subscription(
        &self,
        topic_filter: TopicFilter,
        client_id: ClientId,
    ) {
        let mut write_handle = self.write_handle.lock();
        let operation = RemoveSubscription(topic_filter, client_id);
        write_handle.append(operation);
        write_handle.publish();
    }

    pub fn get_subscriptions(&self, publish_topic: &TopicName) -> Vec<Subscriber> {
        let a = self.read_handle.enter().unwrap();
        a.get_subscriptions(publish_topic)
    }
}
