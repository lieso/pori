use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct LoadingContext {
    pub global_tokens: u64,
    pub stage_tokens: HashMap<String, u64>,
    pub stage_messages: Vec<(String, Vec<StageMessage>)>,
}

#[derive(Clone, Debug)]
pub struct StageMessage {
    pub message: String,
    pub tokens: u64,
}

impl LoadingContext {
    pub fn new() -> Self {
        Self {
            global_tokens: 0,
            stage_tokens: HashMap::new(),
            stage_messages: Vec::new(),
        }
    }
}
