//! # OmniBox
//! `OmniBox` is a library that provides an opinionated testing environment for chain abstracted applications.
//! It contains a set of utilities and encapsulates multiple test environments into a unified framework that makes developing chain abstracted applications easier than ever.
//! Features
//! - Support for EVM Chains, Bitcoin and NEAR
//! - Pre configured accounts for easy development
//! - Account creation
//! - Utilities for account derivation and signature construction
//! - Utilities to interact with the NEAR contract via a friendly NEAR JSON RPC client
//! - Automatic compilation and deployment
mod account_config;
mod chain_config;
mod clients;
mod contexts;
pub mod friendly_near_json_rpc_client;
mod network;
mod omni_box;
mod omni_box_options;
pub mod utils;

pub use omni_box::OmniBox;

use account_config::near_account::NearAccount;
use account_config::Account;
