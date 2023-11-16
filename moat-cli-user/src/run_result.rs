// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::ser::serializers::AllocSerializer;
use std::fmt;
use std::ops::Range;
use zk_citadel::license::License;
// use rkyv::{check_archived_root, Archive, Deserialize, Infallible, Serialize};
use sha3::{Digest, Sha3_256};

pub struct SubmitRequestSummary {
    pub psk_lp: String,
    pub tx_id: String,
    pub request_hash: String,
}

pub struct UseLicenseSummary {
    pub license_blob: Vec<u8>,
    pub tx_id: String,
    pub session_cookie: String,
    pub user_attr: String,
    pub session_id: String,
}

pub struct LicenseContractSummary {
    pub num_licenses: u32,
    pub num_sessions: u32,
}

#[allow(clippy::large_enum_variant)]
/// Possible results of running a command in interactive mode
pub enum RunResult {
    SubmitRequest(SubmitRequestSummary),
    ListLicenses(Range<u64>, Vec<(License, bool)>),
    UseLicense(Option<UseLicenseSummary>),
    ShowState(LicenseContractSummary),
    Empty,
}

impl fmt::Display for RunResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RunResult::*;
        match self {
            SubmitRequest(summary) => {
                writeln!(
                    f,
                    "submitting request to provider psk: {}",
                    summary.psk_lp
                )?;
                writeln!(
                    f,
                    "request submitting transaction {} confirmed",
                    summary.tx_id
                )?;
                writeln!(f, "request submitted: {}", summary.request_hash)?;
                Ok(())
            }
            ListLicenses(block_range, licenses) => {
                writeln!(
                    f,
                    "getting licenses within the block height range {:?}:",
                    block_range
                )?;
                if licenses.is_empty() {
                    writeln!(f, "licenses not found")?;
                } else {
                    for (license, is_owned) in licenses.iter() {
                        writeln!(
                            f,
                            "license: {} {}",
                            RunResult::to_hash_hex(license),
                            if *is_owned { "owned" } else { "" }
                        )?;
                    }
                }
                Ok(())
            }
            UseLicense(summary) => {
                match summary {
                    Some(summary) => {
                        writeln!(
                            f,
                            "using license: {}",
                            Self::blob_to_hash_hex(
                                summary.license_blob.as_slice()
                            )
                        )?;
                        writeln!(
                            f,
                            "use license executing transaction {} confirmed",
                            summary.tx_id
                        )?;
                        writeln!(f)?;
                        writeln!(
                            f,
                            "license {} used",
                            Self::blob_to_hash_hex(
                                summary.license_blob.as_slice()
                            ),
                        )?;
                        writeln!(f)?;
                        writeln!(
                            f,
                            "session cookie: {}",
                            summary.session_cookie
                        )?;
                        writeln!(f)?;
                        writeln!(f, "user attributes: {}", summary.user_attr)?;
                        writeln!(f, "session id: {}", summary.session_id)?;
                    }
                    _ => {
                        writeln!(f, "Please obtain a license")?;
                    }
                }
                Ok(())
            }
            ShowState(summary) => {
                writeln!(
                    f,
                    "license contract state - licenses: {}, sessions: {}",
                    summary.num_licenses, summary.num_sessions
                )?;
                Ok(())
            }
            Empty => Ok(()),
        }
    }
}

impl RunResult {
    pub fn to_hash_hex<T>(object: &T) -> String
    where
        T: rkyv::Serialize<AllocSerializer<16386>>,
    {
        let blob = rkyv::to_bytes::<_, 16386>(object)
            .expect("type should serialize correctly")
            .to_vec();
        Self::blob_to_hash_hex(blob.as_slice())
    }

    pub fn blob_to_hash_hex(blob: &[u8]) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(blob);
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn to_blob_hex<T>(object: &T) -> String
    where
        T: rkyv::Serialize<AllocSerializer<16386>>,
    {
        let blob = Self::to_blob(object);
        hex::encode(blob)
    }

    pub fn to_blob<T>(object: &T) -> Vec<u8>
    where
        T: rkyv::Serialize<AllocSerializer<16386>>,
    {
        rkyv::to_bytes::<_, 16386>(object)
            .expect("type should serialize correctly")
            .to_vec()
    }
}
