// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_bytes::{DeserializableSlice, Serializable};
use dusk_pki::SecretSpendKey;
use moat_core::Error;

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
