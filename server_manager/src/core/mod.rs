//! Core system components for Server Manager.
//!
//! This module provides hardware detection, Docker management, UFW firewall configuration,
//! configuration persistence, secret management, system optimization, and user administration.

pub mod compose;
pub mod config;
pub mod docker;
pub mod firewall;
pub mod hardware;
pub mod secrets;
pub mod system;
pub mod users;
