// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_bytes::{DeserializableSlice, Serializable};
use dusk_pki::SecretSpendKey;
use moat_core::{Error, RequestCreator};
use rand::rngs::StdRng;
use rand::SeedableRng;
use sha2::Digest;
use sha2::Sha256;

#[test]
#[ignore]
fn ssk_to_vk() -> Result<(), Error> {
    let ssk_hex = "fd611dc2cfe15488e3cb94b410fadd3a5e77057be64574eb9b6acaf967a37d0514d0ce88727a24d3756a08bb8ae072d8aaaa88f88768c8a9487fb50678ba5204";
    let ssk_bytes = hex::decode(ssk_hex)?;
    let ssk = SecretSpendKey::from_slice(ssk_bytes.as_slice())?;
    let vk = ssk.view_key();
    println!("vk={}", hex::encode(vk.to_bytes()));
    Ok(())
}

#[test]
#[ignore]
fn create_serialized_request() -> Result<(), Error> {
    let rng = &mut StdRng::seed_from_u64(0xcafe);
    let request = RequestCreator::create_from_hex_args(
        "c6afd78c8b3902b474d4c0972b62888e4b880dccf8da68e86266fefa45ee7505926f06ab82ac200995f1239d518fdb74903f225f4460d8db62f2449f6d4dc402",
        "29c4336ef24e585f4506e32e269c5363a71f7dcd74586b210c56e569ad2644e832c785f102dd3c985c705008ec188be819bac85b65c9f70decb9adcf4a72cc43",
        rng,
    )?;
    let v = rkyv::to_bytes::<_, 8192>(&request)
        .expect("Infallible")
        .to_vec();
    println!("request={}", hex::encode(v));
    Ok(())
}

#[test]
#[ignore]
fn encode_password() -> Result<(), Error> {
    const PSW: &str = "password";
    let mut hasher = Sha256::new();
    hasher.update(PSW.as_bytes());
    println!("password={}", hex::encode(hasher.finalize().to_vec()));
    Ok(())
}

#[test]
#[ignore]
fn encode_password_old() -> Result<(), Error> {
    const PSW: &str = "password";
    let hash = blake3::hash(PSW.as_bytes());
    println!("password={}", hex::encode(hash.as_bytes()));
    Ok(())
}
