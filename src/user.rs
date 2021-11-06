use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub password: String,
    pub ip: Option<String>,
    pub if_name: Option<String>,
}

impl User {
    pub fn new(username: String, password: String, ip: String) -> Self {
        Self {
            username,
            password,
            ip: Some(ip),
            if_name: None,
        }
    }

    pub fn new_with_if_name(username: String, password: String, if_name: String) -> Self {
        Self {
            username,
            password,
            ip: None,
            if_name: Some(if_name),
        }
    }
}
