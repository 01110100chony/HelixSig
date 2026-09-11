//! Linux-first signal-processing experiment. The native kernel remains independent.
pub mod bridge;
mod concurrent;
pub mod config;
mod control;
pub mod event;
pub mod metrics;
pub mod output;
pub mod processing;
pub mod runtime;
pub mod source;
