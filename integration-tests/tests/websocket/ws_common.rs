// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

/// A request sent by the websocket client, asking for a specific contract
/// function to be executed with the given arguments.
#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct ExecutionRequest {
    /// The request ID, allowing for differentiating multiple in-flight
    /// requests.
    pub request_id: Option<i32>,
    /// The contract to call.
    #[serde_as(as = "serde_with::hex::Hex")]
    pub contract: [u8; 32],
    /// The function name to call in the contract.
    pub fn_name: String,
    /// The arguments to pass to the function.
    #[serde_as(as = "serde_with::hex::Hex")]
    pub fn_args: Vec<u8>,
}

/// Response to a [`ExecutionRequest`] with the same `request_id`.
#[serde_with::serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct ExecutionResponse {
    /// The request ID, allowing for differentiating multiple in-flight
    /// requests.
    pub request_id: Option<i32>,
    /// The data returned by the contract call.
    #[serde_as(as = "serde_with::hex::Hex")]
    pub data: Vec<u8>,
    /// A possible error happening during the contract call.
    pub error: Option<String>,
}

#[derive(Debug)]
pub enum ExecutionError {
    Json(serde_json::Error),
}

impl Display for ExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::Json(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<serde_json::Error> for ExecutionError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}
