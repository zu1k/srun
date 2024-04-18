pub use file::read_config_from_file;
pub use srun::*;
pub use user::User;
pub use utils::{get_ip_by_if_name, select_ip};
pub use xencode::param_i;

mod file;
#[cfg(feature = "ureq")]
mod http_client;
mod srun;
mod user;
mod utils;
mod xencode;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
