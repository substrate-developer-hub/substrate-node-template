//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;
// mod command_helper;
mod rpc;

use node_template_runtime::{FramelessCall, FramelessTransaction};
use sp_core::crypto::Pair;
use sp_core::Encode;
use sp_core::{H256, H512};
use sp_runtime::traits::Extrinsic;

fn main() -> sc_cli::Result<()> {
	println!("Here's some bytes to use as a transaction");

	// Basic call information
	let call = FramelessCall::Toggle;
	let salt = 3u8;
	let data_to_sign = (&call, &salt).encode();

	// Sign by alice
	let alice_pair = match sp_core::sr25519::Pair::from_string(&String::from("//Alice"), None) {
		Ok(pair) => pair,
		Err(_) => {
			println!("failed making alice's pair");
			return Ok(())
		}
	};
	let alice_public_h256 = H256::from_slice(&alice_pair.public().as_ref());
	let signature = alice_pair.sign(&data_to_sign);
	let signature_h512 = H512::from_slice(&signature.as_ref());

	// Construct the transaction
	let tx = FramelessTransaction::new(
		call,
		Some((
			salt,
			signature_h512,
			alice_public_h256,
		))
	).expect("I've given valid data, please construct a valid transaction for me :pray:");

	println!("Encoded final transaction:");
	println!("{:?}", tx);
	println!("{:?}", hex::encode(&tx.encode()));

	println!("\n\n\n\n\n");
	command::run()
}
