use ckb_std::ckb_types::{
    bytes::Bytes,
    packed::{Byte32Vec, Script, Transaction},
};
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use serde::{Deserialize, Serialize};
use serde_molecule::dynvec_serde;

/// User-Defined Token (UDT) trait for implementing custom tokens on CKB
///
/// This trait defines the standard interface for implementing fungible tokens
/// on the CKB blockchain following the SSRI protocol. 
///
/// # Implementation Notes
///
/// - All amounts are represented as u128 in convention
/// - Methods that modify state return a Transaction object for further processing in CCC.
/// - Verification methods are separate from state-changing methods
/// - All methods are optional to implement: you return SSRIError::SSRIMethodsNotImplemented if you don't implement a method.
///
/// # Example
///
/// ```rust,no_run
/// use ckb_ssri_sdk::public_module_traits::udt::UDT;
///
/// struct MyToken;
///
/// impl UDT for MyToken {
///     type Error = ();
///     
///     fn decimals() -> Result<u8, Self::Error> {
///         // Implementation
///         Ok(6u8)
///     }
///     // ... implement other required methods
/// }
/// ```
pub trait UDT {
    type Error;
    fn transfer(
        tx: Option<Transaction>,
        to_lock_vec: Vec<Script>,
        to_amount_vec: Vec<u128>,
    ) -> Result<Transaction, Self::Error>;
    fn verify_transfer() -> Result<(), Self::Error>;
    fn name() -> Result<Bytes, Self::Error>;
    fn symbol() -> Result<Bytes, Self::Error>;
    fn decimals() -> Result<u8, Self::Error>;
    fn icon() -> Result<Bytes, Self::Error>;
    fn mint(
        tx: Option<Transaction>,
        to_lock_vec: Vec<Script>,
        to_amount_vec: Vec<u128>,
    ) -> Result<Transaction, Self::Error>;
    fn verify_mint() -> Result<(), Self::Error>;
}
pub const UDT_LEN: usize = 16;
pub enum UDTError {
    InsufficientBalance,
    NoMintPermission,
    NoBurnPermission,
}

pub trait UDTPausable: UDT {
    fn pause(
        tx: Option<Transaction>,
        lock_hashes: &Vec<[u8; 32]>,
    ) -> Result<Transaction, Self::Error>;
    fn unpause(
        tx: Option<Transaction>,
        lock_hashes: &Vec<[u8; 32]>,
    ) -> Result<Transaction, Self::Error>;
    fn is_paused(lock_hashes: &Vec<[u8; 32]>) -> Result<Vec<bool>, Self::Error>;
    fn enumerate_paused(offset: u64, limit: u64) -> Result<Byte32Vec, Self::Error>;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UDTPausableData {
    pub pause_list: Vec<[u8; 32]>,
    pub next_type_script: Option<ScriptLike>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScriptLike {
    pub code_hash: [u8; 32],
    pub hash_type: u8,
    pub args: Vec<u8>,
}
pub enum UDTPausableError {
    NoPausePermission,
    NoUnpausePermission,
    AbortedFromPause,
    IncompletePauseList,
    CyclicPauseList,
}
