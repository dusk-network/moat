// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fmt;

pub struct SessionSummary {
    pub session_id: String,
    pub session: Vec<String>,
}

pub struct LicenseContractSummary {
    pub num_licenses: u32,
    pub num_sessions: u32,
}

#[allow(clippy::large_enum_variant)]
/// Possible results of running a command in interactive mode
pub enum RunResult {
    RequestService,
    GetSession(Option<SessionSummary>),
    ShowState(LicenseContractSummary),
}

impl fmt::Display for RunResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RunResult::*;
        match self {
            GetSession(summary) => {
                match summary {
                    Some(summary) => {
                        writeln!(
                            f,
                            "obtained session with id={}:",
                            summary.session_id
                        )?;
                        for s in summary.session.iter() {
                            writeln!(f, "{}", s)?;
                        }
                    }
                    _ => {
                        writeln!(f, "session not found")?;
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
            RequestService => Ok(()),
        }
    }
}
