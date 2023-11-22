// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::ser::serializers::AllocSerializer;
use std::fmt;
use std::ops::Range;
use zk_citadel::license::{License, Request};
// use rkyv::{check_archived_root, Archive, Deserialize, Infallible, Serialize};
use sha3::{Digest, Sha3_256};

pub struct RequestsLPSummary {
    pub found_total: usize,
    pub found_owned: usize,
}

pub struct IssueLicenseSummary {
    pub request: Request,
    pub tx_id: String,
    pub license_blob: Vec<u8>,
}

pub struct LicenseContractSummary {
    pub num_licenses: u32,
    pub num_sessions: u32,
}

#[allow(clippy::large_enum_variant)]
/// Possible results of running a command in interactive mode
pub enum RunResult {
    RequestsLP(RequestsLPSummary, Vec<Request>),
    IssueLicense(Option<IssueLicenseSummary>),
    ListLicenses(Range<u64>, Vec<License>),
    ShowState(LicenseContractSummary),
}

impl fmt::Display for RunResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RunResult::*;
        match self {
            RequestsLP(summary, requests) => {
                writeln!(
                    f,
                    "found {} requests total, {} requests for this LP:",
                    summary.found_total, summary.found_owned
                )?;
                for request in requests.iter() {
                    writeln!(
                        f,
                        "request to process by LP: {}",
                        RunResult::to_hash_hex(request)
                    )?;
                }
                Ok(())
            }
            IssueLicense(summary) => match summary {
                Some(summary) => {
                    writeln!(
                        f,
                        "issuing license for request: {}",
                        RunResult::to_hash_hex(&summary.request)
                    )?;
                    writeln!(
                        f,
                        "license issuing transaction {} confirmed",
                        summary.tx_id
                    )?;
                    writeln!(
                        f,
                        "issued license: {}",
                        RunResult::blob_to_hash_hex(
                            summary.license_blob.as_slice()
                        )
                    )?;
                    Ok(())
                }
                _ => {
                    writeln!(f, "Request not found")?;
                    Ok(())
                }
            },
            ListLicenses(block_range, licenses) => {
                writeln!(
                    f,
                    "getting licenses within the block height range {:?}:",
                    block_range
                )?;
                if licenses.is_empty() {
                    writeln!(f, "licenses not found")?;
                } else {
                    for license in licenses.iter() {
                        writeln!(
                            f,
                            "license: {}",
                            RunResult::to_hash_hex(license),
                        )?;
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
        }
    }
}

impl RunResult {
    pub fn to_hash_hex<T>(object: &T) -> String
    where
        T: rkyv::Serialize<AllocSerializer<16386>>,
    {
        let blob = rkyv::to_bytes::<_, 16386>(object)
            .expect("Serializing should be infallible")
            .to_vec();
        Self::blob_to_hash_hex(blob.as_slice())
    }

    pub fn blob_to_hash_hex(blob: &[u8]) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(blob);
        let result = hasher.finalize();
        hex::encode(result)
    }
}
