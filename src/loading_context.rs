use std::collections::HashMap;

pub struct LoadingContext {
    pub global_tokens: u64,
    pub stage_tokens: HashMap<String, u64>,
}

impl LoadingContext {
    pub fn new() -> Self {
        Self {
            global_tokens: 0,
            stage_tokens: HashMap::new(),
        }
    }
}
