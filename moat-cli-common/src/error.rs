// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::borrow::Cow;
use std::num::ParseIntError;
use std::sync::Arc;
use thiserror::Error;

// todo: rename CliError to Error
// todo: make sure it is used in other CLIs as well

#[derive(Error, Debug, Clone)]
pub enum Error {
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
    NotFound(Cow<'static, str>),
    /// Invalid entry
    #[error("Invalid entry: {0:?}")]
    InvalidEntry(Cow<'static, str>),
    /// Invalid config value
    #[error("Invalid config value: {0:?}")]
    InvalidConfigValue(Cow<'static, str>),
}

impl From<moat_core::Error> for Error {
    fn from(e: moat_core::Error) -> Self {
        Error::Moat(Arc::from(e))
    }
}

impl From<requestty::ErrorKind> for Error {
    fn from(e: requestty::ErrorKind) -> Self {
        Error::Interaction(Arc::from(e))
    }
}

impl From<clap::error::ErrorKind> for Error {
    fn from(e: clap::error::ErrorKind) -> Self {
        Error::Parsing(Arc::from(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(Arc::from(e))
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::InvalidEntry(e.to_string().into())
    }
}
