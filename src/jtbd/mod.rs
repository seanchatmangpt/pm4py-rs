//! JTBD (Jobs-to-be-Done) YAWL v6 runner module.
//!
//! Provides checkpoint span emission for the YAWL v6 workflow engine integration.

pub mod yawlv6_runner;

pub use yawlv6_runner::emit_checkpoint;
