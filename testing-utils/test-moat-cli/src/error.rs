// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::sync::Arc;

#[derive(Debug)]
pub enum CliError {
    /// Moat core error
    Moat(Arc<moat_core::Error>),
    /// Interaction error
    Interaction(Arc<requestty::ErrorKind>),
    /// Parsing error
    Parsing(Arc<clap::error::ErrorKind>),
    /// IO Error
    IO(Arc<std::io::Error>),
}

impl From<moat_core::Error> for CliError {
    fn from(e: moat_core::Error) -> Self {
        CliError::Moat(Arc::from(e))
    }
}

impl From<requestty::ErrorKind> for CliError {
    fn from(e: requestty::ErrorKind) -> Self {
        CliError::Interaction(Arc::from(e))
    }
}

impl From<clap::error::ErrorKind> for CliError {
    fn from(e: clap::error::ErrorKind) -> Self {
        CliError::Parsing(Arc::from(e))
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::IO(Arc::from(e))
    }
}
