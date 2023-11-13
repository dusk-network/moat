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
    JsonParsing(Arc<serde_json::Error>),
    #[error(transparent)]
    IO(Arc<std::io::Error>),
    #[error(transparent)]
    DuskWallet(Arc<dusk_wallet::Error>),
    #[error("A serialization error occurred: {0:?}")]
    Bytes(Arc<dusk_bytes::Error>),
    #[error("A serialization error occurred.")]
    Rkyv,
    #[error(transparent)]
    Hex(Arc<hex::FromHexError>),
    #[error("A GraphQL error occurred: {0:?}")]
    GQL(Box<str>),
    #[error("TransactionNotFound")]
    TransactionNotFound,
    #[error("A base64 decode error occurred: {0:?}")]
    Base64Decode(Arc<base64::DecodeError>),
    #[error(transparent)]
    WebSocket(Arc<tokio_tungstenite::tungstenite::Error>),
    #[error("WebSocketStreamClosed")]
    WebSocketStreamClosed,
    #[error("Invalid query response: {0:?}")]
    InvalidQueryResponse(Box<str>),
    #[error("Transaction error: {0:?}")]
    Transaction(Box<str>),
    #[error("Stream item not present or stream error: {0:?}")]
    Stream(Box<str>),
    #[error("A PLONK error occurred: {0:?}")]
    Plonk(Arc<dusk_plonk::error::Error>),
    #[error("A CRS error occurred: {0:?}")]
    CRS(Box<str>),
    #[error(transparent)]
    HttpClient(Arc<reqwest::Error>),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::JsonParsing(Arc::from(e))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(Arc::from(e))
    }
}

impl From<dusk_wallet::Error> for Error {
    fn from(e: dusk_wallet::Error) -> Self {
        Error::DuskWallet(Arc::from(e))
    }
}

impl From<dusk_bytes::Error> for Error {
    fn from(e: dusk_bytes::Error) -> Self {
        Error::Bytes(Arc::from(e))
    }
}

impl From<hex::FromHexError> for Error {
    fn from(e: hex::FromHexError) -> Self {
        Error::Hex(Arc::from(e))
    }
}

impl From<base64::DecodeError> for Error {
    fn from(e: base64::DecodeError) -> Self {
        Error::Base64Decode(Arc::from(e))
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
        Error::WebSocket(Arc::from(e))
    }
}

impl From<dusk_plonk::error::Error> for Error {
    fn from(e: dusk_plonk::error::Error) -> Self {
        Error::Plonk(Arc::from(e))
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::HttpClient(Arc::from(e))
    }
}
