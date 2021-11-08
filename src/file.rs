use crate::User;
use serde::Deserialize;
use std::{collections::LinkedList, error::Error, fs::File, io::BufReader, path::Path};

#[derive(Deserialize, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub server: Option<String>,
    pub strict_bind: bool,
    users: LinkedList<User>,
}

impl Iterator for Config {
    type Item = User;

    fn next(&mut self) -> Option<Self::Item> {
        self.users.pop_front()
    }
}

pub fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}
