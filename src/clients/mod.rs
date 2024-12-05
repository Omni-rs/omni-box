mod anvil;
mod bitcoin_core;
mod near;

pub use anvil::get_anvil_instance;
pub use bitcoin_core::get_bitcoin_instance;
pub use near::get_near_instance;
