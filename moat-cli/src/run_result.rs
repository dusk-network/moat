// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::ser::serializers::AllocSerializer;
use std::fmt;
use zk_citadel::license::Request;
// use rkyv::{check_archived_root, Archive, Deserialize, Infallible, Serialize};
use sha3::{Digest, Sha3_256};

pub struct SubmitRequestSummary {
    pub psk_lp: String,
    pub tx_id: String,
    pub request_hash: String,
}

pub struct RequestsSummary {
    pub height: u64,
    pub found_total: usize,
    pub found_owned: usize,
}

pub struct RequestsLPSummary {
    pub found_total: usize,
    pub found_owned: usize,
}

pub struct IssueLicenseSummary {
    pub request: Request,
    pub tx_id: String,
    pub license_blob: Vec<u8>,
}

#[allow(clippy::large_enum_variant)]
/// Possible results of running a command in interactive mode
pub enum RunResult {
    SubmitRequest(SubmitRequestSummary),
    Requests(RequestsSummary, Vec<Request>),
    RequestsLP(RequestsLPSummary, Vec<Request>),
    IssueLicense(Option<IssueLicenseSummary>),
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
            Requests(summary, requests) => {
                writeln!(
                    f,
                    "scanned {} blocks, found {} requests, {} owned requests:",
                    summary.height, summary.found_total, summary.found_owned,
                )?;
                for request in requests.iter() {
                    writeln!(f, "request: {}", Self::to_hash_hex(request))?;
                }
                Ok(())
            }
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
        let blob = rkyv::to_bytes::<_, 16386>(object)
            .expect("type should serialize correctly")
            .to_vec();
        hex::encode(blob)
    }
}
