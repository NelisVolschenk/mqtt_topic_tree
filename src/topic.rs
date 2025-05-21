use std::sync::Arc;

/// A Struct for searching through the topic tree
#[derive(Clone)]
pub struct TopicName {
    pub(crate) length: usize,
    pub(crate) topic_indices: Vec<(usize, usize)>,
    pub(crate) orig_str: Arc<String>,
}

impl TopicName {
    pub fn get_part(&self, index: usize) -> Option<&str> {
        if index >= self.length {
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
        let str_max_chars = value.len();
        if str_max_chars == 0 {
            return Err(TopicNameError::IsEmpty);
        };
        if str_max_chars > u16::MAX as usize {
            return Err(TopicNameError::TooLong);
        };
        let mut topic_indices = Vec::with_capacity(str_max_chars);
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
        topic_indices.push((prev_slice, str_max_chars));
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
    pub(crate) length: usize,
    pub(crate) shared_group_name: Option<String>,
    pub(crate) topic_indices: Vec<(usize, usize)>,
    pub(crate) orig_str: Arc<String>,
}

impl TopicFilter {
    pub fn get_part(&self, index: usize) -> Option<&str> {
        if index >= self.length {
            return None;
        };
        let startidx = self.topic_indices[index].0;
        let endindex = self.topic_indices[index].1;
        let ret = &self.orig_str[startidx..endindex];
        Some(ret)
    }
}

impl TryFrom<String> for TopicFilter {
    type Error = TopicFilterError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let orig_str = Arc::new(value.clone());
        let str_max_chars = value.len();
        if str_max_chars == 0 {
            return Err(TopicFilterError::IsEmpty);
        };
        if str_max_chars > u16::MAX as usize {
            return Err(TopicFilterError::TooLong);
        };
        let mut topic_indices = Vec::with_capacity(str_max_chars);
        let mut prev_slice = 0;
        for (idx, c) in value.char_indices() {
            if c == '\0' {
                return Err(TopicFilterError::ContainsNull);
            };
            if c == '/' {
                topic_indices.push((prev_slice, idx));
                prev_slice = idx + 1;
            }
        }
        topic_indices.push((prev_slice, str_max_chars));
        // Check shared subscription
        let mut shared_group_name = None;
        let startidx = topic_indices[0].0;
        let endindex = topic_indices[0].1;
        let first_level = &value[startidx..endindex];
        if first_level == "$share" {
            let startidx = topic_indices[1].0;
            let endindex = topic_indices[1].1;
            shared_group_name = Some(value[startidx..endindex].to_owned());
            topic_indices = topic_indices[2..].iter().map(|x| *x).collect();
        }
        let length = topic_indices.len();

        Ok(Self {
            length,
            shared_group_name,
            topic_indices,
            orig_str
        })
    }
}

#[derive(Debug)]
pub enum TopicFilterError {
    ContainsNull,
    IsEmpty,
    TooLong,
}
