#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;

mod srun;
pub use srun::*;

mod xencode;
pub use xencode::param_i;

mod file;
pub use file::read_config_from_file;

mod utils;
pub use utils::select_ip;
