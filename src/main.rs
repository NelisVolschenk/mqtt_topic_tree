mod topic_tree;
mod topic;

use std::time::Instant;
use left_right;
use crate::topic::{QoS, TopicFilter, TopicName};
use crate::topic_tree::TopicTree;


fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(run_topic_tree())
}
async fn run_topic_tree() {
    println!("Running main");
    let mut t = TopicTree::default();
    let s1 = TopicFilter::try_from("home/+/+".to_owned()).unwrap();
    t.add_subscription(s1, 1, QoS::Level0);
    let s2 = TopicFilter::try_from("home/#".to_owned()).unwrap();
    t.add_subscription(s2, 2, QoS::Level0);
    let s3 = TopicFilter::try_from("home/+/#".to_owned()).unwrap();
    t.add_subscription(s3, 3, QoS::Level0);
    let static_t: &'static mut TopicTree = Box::leak(Box::new(t));
    let top = "home/bedroom/light";
    let t1 = Instant::now();
    let topic = TopicName::try_from(top.to_owned()).unwrap();
    let tdelta = (Instant::now() - t1).as_nanos();
    println!("Topic Creation took {tdelta} ns");
    let num_ops = 1_000_000;
    let num_threads = 1;
    let mut tasks = Vec::with_capacity(num_threads as usize);
    let t_start = Instant::now();
    for _ in 0..num_threads{
        tasks.push(
            tokio::spawn(
                operation(static_t, num_ops, top)
            )
        );
    }
    futures::future::join_all(tasks).await;
    let t_end = Instant::now();
    let t_delta = (t_end - t_start).as_nanos()/(num_ops as u128 * num_threads);
    println!("Lookup took {t_delta} ns per iteration with {num_threads} threads");
}

async fn operation(topictree: &TopicTree, num_ops: u32, top: &str) {

    for i in 0..num_ops {
        let topic = TopicName::try_from(top.to_owned()).unwrap();
        let ids = topictree.get_routes(&topic);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::topic::{QoS, TopicFilter, TopicName};
    use crate::topic_tree::{ClientId, TopicTree};

    #[test]
    fn test_add_sub() {
        let mut t = TopicTree::default();
        let s1 = TopicFilter::try_from("home/+/+".to_owned()).unwrap();
        t.add_subscription(s1, 1, QoS::Level0);
        let s2 = TopicFilter::try_from("home/#".to_owned()).unwrap();
        t.add_subscription(s2, 2, QoS::Level0);
        let s3 = TopicFilter::try_from("home/+/#".to_owned()).unwrap();
        t.add_subscription(s3, 3, QoS::Level0);
        let s4 = TopicFilter::try_from("home/bedroom/light".to_owned()).unwrap();
        t.add_subscription(s4, 4, QoS::Level0);
        // println!("{:#?}", t);
        let topic = TopicName::try_from("home/bedroom/light".to_owned()).unwrap();
        let ids = t.get_routes(&topic);
        let client_ids: Vec<ClientId> = ids.iter().map(|x|x.client_id).collect();
        for i in 1..5 {
            assert!(client_ids.contains(&i))
        }
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
        for i in 0..num_ops {

            let ids = t.get_routes(&topic);
        }
        let t_end = Instant::now();
        let t_delta = (t_end - t_start).as_nanos()/num_ops as u128;
        println!("Lookup took {t_delta} ns per iteration");
    }
}
