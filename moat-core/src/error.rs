// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error(transparent)]
    JsonParsingError(Arc<serde_json::Error>),
    #[error(transparent)]
    FileError(Arc<std::io::Error>),
    #[error(transparent)]
    DuskWalletError(Arc<dusk_wallet::Error>),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::JsonParsingError(Arc::from(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::FileError(Arc::from(e))
    }
}

impl From<dusk_wallet::Error> for Error {
    fn from(e: dusk_wallet::Error) -> Self {
        Error::DuskWalletError(Arc::from(e))
    }
}
