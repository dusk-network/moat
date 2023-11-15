// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::sync::Arc;
use thiserror::Error;

// todo: rename CliError to Error
// todo: make sure it is used in other CLIs as well

#[derive(Error, Debug, Clone)]
pub enum CliError {
    /// Moat core error
    #[error(transparent)]
    Moat(Arc<moat_core::Error>),
    /// Interaction error
    #[error(transparent)]
    Interaction(Arc<requestty::ErrorKind>),
    /// Parsing error
    #[error("Parsing error occurred: {0:?}")]
    Parsing(Arc<clap::error::ErrorKind>),
    /// IO Error
    #[error(transparent)]
    IO(Arc<std::io::Error>),
    /// Not found error
    #[error("Not found: {0:?}")]
    NotFound(Box<str>),
    /// Hex string error
    #[error("Incorrect hexadecimal string: {0:?}")]
    HexString(Box<str>),
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

impl From<hex::FromHexError> for CliError {
    fn from(e: hex::FromHexError) -> Self {
        CliError::HexString(Box::from(e.to_string()))
    }
}

impl From<dusk_bytes::Error> for CliError {
    fn from(e: dusk_bytes::Error) -> Self {
        CliError::Moat(Arc::from(moat_core::Error::Bytes(Arc::from(e))))
    }
}
