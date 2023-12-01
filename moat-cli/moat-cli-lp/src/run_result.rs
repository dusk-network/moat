// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fmt;
use std::ops::Range;
use zk_citadel::license::{License, Request};
use zk_citadel_moat::MoatCoreUtils;

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
                        MoatCoreUtils::to_hash_hex(request)
                    )?;
                }
                Ok(())
            }
            IssueLicense(summary) => match summary {
                Some(summary) => {
                    writeln!(
                        f,
                        "issuing license for request: {}",
                        MoatCoreUtils::to_hash_hex(&summary.request)
                    )?;
                    writeln!(
                        f,
                        "license issuing transaction {} confirmed",
                        summary.tx_id
                    )?;
                    writeln!(
                        f,
                        "issued license: {}",
                        MoatCoreUtils::blob_to_hash_hex(
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
                            MoatCoreUtils::to_hash_hex(license),
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
