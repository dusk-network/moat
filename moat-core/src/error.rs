// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Request not present or incorrect: {0:?}")]
    RequestNotPresent(Box<str>),
    #[error(transparent)]
    JsonParsingError(Arc<serde_json::Error>),
    #[error(transparent)]
    FileError(Arc<std::io::Error>),
    #[error(transparent)]
    DuskWalletError(Arc<dusk_wallet::Error>),
    #[error("A serialization error occurred: {0:?}")]
    BytesError(Arc<dusk_bytes::Error>),
    #[error(transparent)]
    HexError(Arc<hex::FromHexError>),
    #[error("A GraphQL error occurred: {0:?}")]
    GQLError(Box<str>),
    #[error("TransactionNotFound")]
    TransactionNotFound,
    #[error("A base64 decode error occurred: {0:?}")]
    Base64DecodeError(Arc<base64::DecodeError>),
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

impl From<dusk_bytes::Error> for Error {
    fn from(e: dusk_bytes::Error) -> Self {
        Error::BytesError(Arc::from(e))
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        Error::HexError(Arc::from(e))
    }
}

impl From<gql_client::GraphQLError> for Error {
    fn from(e: gql_client::GraphQLError) -> Self {
        Error::GQLError(Box::from(e.message()))
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::Base64DecodeError(Arc::from(e))
    }
}
