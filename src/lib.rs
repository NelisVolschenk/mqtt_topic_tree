mod topic_tree;
mod topic;

use left_right;

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::topic::{QoS, TopicFilter, TopicName};
    use crate::topic_tree::TopicTree;

    #[test]
    fn test_add_sub() {
        let mut t = TopicTree::default();
        let s1 = TopicFilter::try_from("home/+/+".to_owned()).unwrap();
        t.add_subscription(s1, 1, QoS::Level0);
        let s2 = TopicFilter::try_from("home/#".to_owned()).unwrap();
        t.add_subscription(s2, 2, QoS::Level0);
        let s3 = TopicFilter::try_from("home/+/#".to_owned()).unwrap();
        t.add_subscription(s3, 3, QoS::Level0);
        println!("{:#?}", t);
        let topic = TopicName::try_from("home/bedroom/light".to_owned()).unwrap();
        let ids = t.get_routes(topic);
        println!("{:?}", ids)
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
        let num_ops = 100_000u64;
        let t_start = Instant::now();
        for i in 0..num_ops {
            let ids = t.get_routes(topic.clone());
        }
        let t_end = Instant::now();
        let t_delta = (t_end - t_start).as_nanos()/num_ops as u128;
        println!("Lookup took {t_delta} ns per iteration");
    }
}
