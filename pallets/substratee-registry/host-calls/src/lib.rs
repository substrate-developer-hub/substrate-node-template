#[cfg(feature = "std")]
mod remote_attestation;

#[cfg(feature = "std")]
use remote_attestation::verify_mra_cert;

use codec::{Decode, Encode};
use runtime_interface::runtime_interface;

#[cfg(feature = "std")]
use log::*;

#[derive(Encode, Decode, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum SgxStatus {
    Invalid,
    Ok,
    GroupOutOfDate,
    GroupRevoked,
    ConfigurationNeeded,
}
impl Default for SgxStatus {
    fn default() -> Self {
        SgxStatus::Invalid
    }
}

#[derive(Encode, Decode, Default, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SgxReport {
    pub mr_enclave: [u8; 32],
    pub pubkey: [u8; 32],
    pub status: SgxStatus,
    pub timestamp: i64,
}

#[runtime_interface]
pub trait RuntimeInterfaces {
    // Only types that implement the RIType (Runtime Interface Type) trait can be returned
    fn verify_ra_report(cert_der: &[u8], signer_attn: &[u32], signer: &[u8]) -> Option<Vec<u8>> {
        debug!("calling into host call verify_ra_cert()");
        match verify_mra_cert(cert_der, signer_attn, signer) {
            Ok(rep) => Some(rep),
            Err(_) => None,
        }
    }
}
