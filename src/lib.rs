pub use file::read_config_from_file;
pub use srun::*;
pub use user::User;
pub use utils::get_ip_by_if_name;
pub use utils::select_ip;
pub use xencode::param_i;

mod user;
mod srun;
mod xencode;
mod file;
mod utils;
#[cfg(feature = "ureq")]
mod http_client;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;