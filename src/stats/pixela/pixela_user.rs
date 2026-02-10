use serde::{Deserialize, Serialize};

use super::subjects::Subject;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PixelaUser {
    username: String,
    token: String,
    subjects: Vec<Subject>,
}
impl PixelaUser {
    pub fn new(username: String, token: String) -> PixelaUser {
        PixelaUser {
            username,
            token,
            subjects: Vec::new(),
        }
    }
    pub fn validate_not_empty(&self) -> bool {
        if self.username().is_empty() {
            return false;
        };
        true
    }
    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn subjects(&self) -> &[Subject] {
        &self.subjects
    }
}
