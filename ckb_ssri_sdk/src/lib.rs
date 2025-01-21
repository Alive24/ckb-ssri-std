#![no_std]
//! # CKB SSRI SDK
//! 
//! A comprehensive framework for implementing SSRI-compliant smart contracts on the Nervos CKB blockchain.
//! 
//! ## Overview
//! 
//! - SSRI stands for `Script Sourced Rich Information`; it is a protocol for strong bindings of relevant information and conventions to the Script itself on CKB. For more information, please read [[EN/CN] Script-Sourced Rich Information - 来源于 Script 的富信息](https://talk.nervos.org/t/en-cn-script-sourced-rich-information-script/8256)>.
//! - For writing CKB Scripts (or "Smart Contracts"), by selectively implementing methods of public module traits (e.g. `UDT`, `UDTExtended`, `UDTPausable`) in combinations, devs would be able to quickly design and organize functionalities that either validate transactions or provide rich information as well as assembling transactions off-chain.
//! - For dApps or other infrastructures that interact with CKB Scripts, you no longer need to retrieve and parse data or assemble transactions by yourself repetitively as they are all provided by SSRI.

//!
//! ## Features
//! 
//! - **Public Traits**: Pre-defined interfaces that receive first-class support within the ecosystem
//! - **Utility Functions**: Helper functions for SSRI-VM syscalls and data handling
//! - **Procedural Macros**: Simplify contract development with automatic SSRI method generation
//!
//! ## Usage
//!
//! Add this to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! ckb_ssri_sdk = "0.1.0"
//! ```
//!
//! ## Example
//! [`pausable-udt`](https://github.com/ckb-devrel/pausable-udt) is a real production level contract (instead of a pseudo-project) that exemplifies the usage of SSRI.

pub mod public_module_traits;
pub mod prelude;
pub mod utils;
pub mod macros;

// Re-export proc macros at crate root for convenience
pub use macros::*;

extern crate alloc;

#[repr(i8)]
#[derive(Debug)]
/// Represents possible errors that can occur during SSRI method execution
/// Should be derivable to `Error` for the actual script to use.
pub enum SSRIError {
    /// The requested SSRI method was not found in the contract
    SSRIMethodsNotFound,
    /// The arguments provided to the SSRI method were invalid
    SSRIMethodsArgsInvalid,
    /// The requested SSRI method is not implemented
    SSRIMethodsNotImplemented,
    /// The method requires a higher execution environment level
    SSRIMethodRequireHigherLevel,
    /// The CKB VM version is not compatible with this implementation
    InvalidVmVersion
}
