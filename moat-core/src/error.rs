// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Payload not present or incorrect: {0:?}")]
    PayloadNotPresent(Box<str>),
    #[error(transparent)]
    JsonParsingError(Arc<serde_json::Error>),
    #[error(transparent)]
    FileError(Arc<std::io::Error>),
    #[error(transparent)]
    DuskWalletError(Arc<dusk_wallet::Error>),
    #[error("A serialization error occurred: {0:?}")]
    BytesError(Arc<dusk_bytes::Error>),
    #[error("A serialization error occurred.")]
    Rkyv,
    #[error(transparent)]
    HexError(Arc<hex::FromHexError>),
    #[error("A GraphQL error occurred: {0:?}")]
    GQLError(Box<str>),
    #[error("TransactionNotFound")]
    TransactionNotFound,
    #[error("A base64 decode error occurred: {0:?}")]
    Base64DecodeError(Arc<base64::DecodeError>),
    #[error(transparent)]
    WebSocketError(Arc<tokio_tungstenite::tungstenite::Error>),
    #[error("WebSocketStreamClosed")]
    WebSocketStreamClosed,
    #[error("Invalid query response: {0:?}")]
    InvalidQueryResponse(Box<str>),
    #[error("Transaction error: {0:?}")]
    TransactionError(Box<str>),
    #[error("Stream item not present or stream error: {0:?}")]
    StreamItem(Box<str>),
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

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::Base64DecodeError(Arc::from(e))
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Error::WebSocketError(Arc::from(e))
    }
}
