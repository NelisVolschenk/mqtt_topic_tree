use std::sync::Arc;

/// A Struct for searching through the topic tree
#[derive(Clone)]
pub struct TopicName {
    pub length: usize,
    pub topic_indices: Vec<(usize, usize)>,
    pub orig_str: Arc<String>,
}

impl TopicName {
    pub fn get_part(&self, index: usize) -> Option<&str> {
        if index > self.length {
            return None;
        };
        let startidx = self.topic_indices[index].0;
        let endindex = self.topic_indices[index].1;
        let ret = &self.orig_str[startidx..endindex];
        Some(ret)
    }
}

impl TryFrom<String> for TopicName {
    type Error = TopicNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let orig_str = Arc::new(value.clone());
        let length = value.len();
        if length == 0 {
            return Err(TopicNameError::IsEmpty);
        };
        if length > u16::MAX as usize {
            return Err(TopicNameError::TooLong);
        };
        let mut topic_indices = Vec::with_capacity(length);
        let mut prev_slice = 0;
        for (idx, c) in value.char_indices() {
            if c == '#' {
                return Err(TopicNameError::ContainsMultiLevelWildcard);
            };
            if c == '+' {
                return Err(TopicNameError::ContiansSingleLevelWildcard);
            };
            if c == '\0' {
                return Err(TopicNameError::ContainsNull);
            };
            if c == '/' {
                topic_indices.push((prev_slice, idx));
                prev_slice = idx + 1;
            }
        }
        topic_indices.push((prev_slice, length));
        let length = topic_indices.len();

        Ok(Self {
            length,
            topic_indices,
            orig_str,
        })
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

#[derive(Clone)]
pub struct TopicFilter {
    pub topic_levels: Vec<String>,
    pub length: usize,
    pub shared_group_name: Option<String>,
}

impl TryFrom<String> for TopicFilter {
    type Error = TopicFilterError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let topic_levels: Vec<String> = value.split("/").map(|x| x.to_owned()).collect();
        let length = topic_levels.len();
        let shared_group_name = None;
        Ok(Self {
            topic_levels,
            length,
            shared_group_name,
        })
    }
}

#[derive(Debug)]
pub enum TopicFilterError {
    ContainsNull,
    IsEmpty,
    TooLong,
}
