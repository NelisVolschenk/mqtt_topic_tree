mod topic_tree;
mod topic;

use left_right;

#[cfg(test)]
mod tests {
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
}
