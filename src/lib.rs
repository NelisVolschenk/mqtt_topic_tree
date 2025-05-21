pub mod sync;
pub mod topic;
pub mod topic_tree;
mod client_types;

pub use crate::sync::MqttTopicTree;
pub use crate::topic_tree::{TopicTree, Subscriber};
pub use crate::topic::{TopicFilter, TopicName};
pub use crate::client_types::{ClientId, QoS};

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::{ClientId, MqttTopicTree, QoS, TopicFilter, TopicName, TopicTree};

    #[test]
    fn test_add_remove_sub() {
        let mut t = TopicTree::default();
        let s1 = TopicFilter::try_from("home/+/+".to_owned()).unwrap();
        t.add_subscription(s1, 1, QoS::Level0);
        let s2 = TopicFilter::try_from("home/#".to_owned()).unwrap();
        t.add_subscription(s2, 2, QoS::Level0);
        let s3 = TopicFilter::try_from("home/+/#".to_owned()).unwrap();
        t.add_subscription(s3, 3, QoS::Level0);
        let s4 = TopicFilter::try_from("$share/group1/home/bedroom/light".to_owned()).unwrap();
        t.add_subscription(s4, 4, QoS::Level0);
        let s5 = TopicFilter::try_from("home/bedroom/light".to_owned()).unwrap();
        t.add_subscription(s5.clone(), 5, QoS::Level0);
        // Test adding 5 subscribers
        let topic = TopicName::try_from("home/bedroom/light".to_owned()).unwrap();
        let ids = t.get_subscriptions(&topic);
        let client_ids: Vec<ClientId> = ids.iter().map(|x| x.client_id).collect();
        for i in 1..=5 {
            assert!(client_ids.contains(&i))
        }
        // Test removing 5
        t.remove_subscription(s5, 5);
        let ids = t.get_subscriptions(&topic);
        let client_ids: Vec<ClientId> = ids.iter().map(|x| x.client_id).collect();
        assert!(!client_ids.contains(&5))
    }

    #[test]
    fn speed_test() {
        let mut t = TopicTree::default();
        let s1 = TopicFilter::try_from("home/+/+".to_owned()).unwrap();
        t.add_subscription(s1, 1, QoS::Level0);
        let s2 = TopicFilter::try_from("home/#".to_owned()).unwrap();
        t.add_subscription(s2, 2, QoS::Level0);
        let s3 = TopicFilter::try_from("home/+/#".to_owned()).unwrap();
        t.add_subscription(s3, 3, QoS::Level0);
        let topic = TopicName::try_from("home/bedroom/light".to_owned()).unwrap();
        let num_ops = 100000;
        let t_start = Instant::now();
        for _ in 0..num_ops {
            let _ = t.get_subscriptions(&topic);
        }
        let t_end = Instant::now();
        let t_delta = (t_end - t_start).as_nanos() / num_ops as u128;
        println!("Topictree Lookup took {t_delta} ns per iteration");
    }

    #[test]
    fn test_sync() {
        let t = MqttTopicTree::default();
        let s1 = TopicFilter::try_from("home/+/+".to_owned()).unwrap();
        t.add_subscription(s1, 1, QoS::Level0);
        let s2 = TopicFilter::try_from("home/#".to_owned()).unwrap();
        t.add_subscription(s2, 2, QoS::Level0);
        let s3 = TopicFilter::try_from("home/+/#".to_owned()).unwrap();
        t.add_subscription(s3, 3, QoS::Level0);
        let topic = TopicName::try_from("home/bedroom/light".to_owned()).unwrap();
        let num_ops = 100000;
        let t_start = Instant::now();
        for _ in 0..num_ops {
            let _ = t.get_subscriptions(&topic);
        }
        let t_end = Instant::now();
        let t_delta = (t_end - t_start).as_nanos() / num_ops as u128;
        println!("MqttTopicTree Lookup took {t_delta} ns per iteration");
    }
}
