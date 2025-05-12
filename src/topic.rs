
/// A Struct for searching through the topic tree
pub struct TopicName {
    pub topic_levels: Vec<String>,
    pub length: usize,
}

impl TopicName {
    fn is_valid(topicname: &str) -> Result<(), TopicNameError> {
        if topicname.len() == 0 {return Err(TopicNameError::IsEmpty)};
        if topicname.len() > u16::MAX as usize {return Err(TopicNameError::TooLong)};
        if topicname.contains("#") {return Err(TopicNameError::ContainsMultiLevelWildcard)};
        if topicname.contains("+") {return Err(TopicNameError::ContiansSingleLevelWildcard)};
        if topicname.contains("\0") {return Err(TopicNameError::ContainsNull)};
        Ok(())
    }
}

impl TryFrom<String> for TopicName {
    type Error = TopicNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match TopicName::is_valid(value.as_str()) {
            Ok(_) => {
                let topic_levels: Vec<String> = value.split("/").map(|x|x.to_owned()).collect();
                let length = topic_levels.len();
                Ok(Self {topic_levels, length})
            }
            Err(e) => {Err(e)}
        }
    }
}

#[derive(Debug)]
pub enum TopicNameError {
    ContainsMultiLevelWildcard,
    ContiansSingleLevelWildcard,
    ContainsNull,
    IsEmpty,
    TooLong,
}

/// Packet delivery [Quality of Service] level.
///
/// [Quality of Service]: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc398718099
#[derive(Clone, Debug)]
pub enum QoS {
    /// `QoS 0`. At most once. No ack needed.
    Level0 = 0,
    /// `QoS 1`. At least once. One ack needed.
    Level1 = 1,
    /// `QoS 2`. Exactly once. Two acks needed.
    Level2 = 2,
}

pub struct TopicFilter {
    pub topic_levels: Vec<String>,
    pub length: usize,
    pub shared_group_name: Option<String>,
}

impl TryFrom<String> for TopicFilter {
    type Error = TopicFilterError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let topic_levels: Vec<String> = value
            .split("/")
            .map(|x| x.to_owned())
            .collect();
        let length = topic_levels.len();
        let shared_group_name = None;
        Ok(Self{
            topic_levels,
            length,
            shared_group_name
        })
    }
}

#[derive(Debug)]
pub enum TopicFilterError {
    ContainsNull,
    IsEmpty,
    TooLong,
}