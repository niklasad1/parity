// Copyright 2015-2019 Parity Technologies (UK) Ltd.
// This file is part of Parity Ethereum.

// Parity Ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity Ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity Ethereum.  If not, see <http://www.gnu.org/licenses/>.

use ethereum_types::{Address, H256};
use lru_cache::LruCache;
use parking_lot::RwLock;
use rlp::encode;

use engines::clique::{ADDRESS_LENGTH, SIGNATURE_LENGTH, VANITY_LENGTH, NULL_NONCE, NULL_MIXHASH};
use error::Error;
use ethkey::{public_to_address, recover as ec_recover, Signature};
use types::header::Header;

/// How many recovered signature to cache in the memory.
pub const CREATOR_CACHE_NUM: usize = 4096;
lazy_static! {
	/// key: header hash
	/// value: creator address
	static ref CREATOR_BY_HASH: RwLock<LruCache<H256, Address>> = RwLock::new(LruCache::new(CREATOR_CACHE_NUM));
}

/// Recover block creator from signature
pub fn recover_creator(header: &Header) -> Result<Address, Error> {
	// Initialization
	let mut cache = CREATOR_BY_HASH.write();

	if let Some(creator) = cache.get_mut(&header.hash()) {
		return Ok(*creator);
	}

	let data = header.extra_data();
	if data.len() < VANITY_LENGTH + SIGNATURE_LENGTH {
		return Err(From::from("extra_data length is not enough!"));
	}

	// Get vanity and signature data from `header.extra_data`
	// Note, this shouldn't have a list of signers because this is only called on `non-checkpoints`
	let (vanity_slice, signature_slice) = data.split_at(data.len() - SIGNATURE_LENGTH);

	// convert `&[u8]` to `[u8; 65]`
	let signature = {
		let mut s = [0; SIGNATURE_LENGTH];
		s.copy_from_slice(signature_slice);
		s
	};

	// modify header and hash it
	let unsigned_header = &mut header.clone();
	unsigned_header.set_extra_data(vanity_slice.to_vec());
	let msg = unsigned_header.hash();

	let pubkey = ec_recover(&Signature::from(signature), &msg)?;
	let creator = public_to_address(&pubkey);

	cache.insert(header.hash(), creator.clone());
	Ok(creator)
}

/// Extract signer list from extra_data.
///
/// Layout of extra_data:
/// ----
/// VANITY: 32 bytes
/// Signers: N * 32 bytes as hex encoded (20 characters)
/// Signature: 65 bytes
/// --
pub fn extract_signers(header: &Header) -> Result<Vec<Address>, Error> {
	let data = header.extra_data();

	println!("extra_data: {}", data.len());
	if data.len() <= VANITY_LENGTH + SIGNATURE_LENGTH {
		return Err(Box::new("Invalid extra_data size.").into());
	}

	// extract only the portion of extra_data which includes the signer list
	let signers_raw = &data[(VANITY_LENGTH)..data.len() - (SIGNATURE_LENGTH)];

	if signers_raw.len() % ADDRESS_LENGTH != 0 {
		return Err(Box::new("bad signer list.").into());
	}

	let num_signers = signers_raw.len() / 20;
	let mut signers_list: Vec<Address> = Vec::with_capacity(num_signers);

	for i in 0..num_signers {
		let mut signer = Address::default();
		signer.copy_from_slice(&signers_raw[i * ADDRESS_LENGTH..(i + 1) * ADDRESS_LENGTH]);
		signers_list.push(signer);
	}

	// NOTE: signers list must be sorted by ascending order.
	signers_list.sort();

	Ok(signers_list)
}

pub fn null_seal() -> Vec<Vec<u8>> {
	vec![encode(&NULL_MIXHASH.to_vec()), encode(&NULL_NONCE.to_vec())]
}


