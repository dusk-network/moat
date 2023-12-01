// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Library for submitting license requests to the Dusk blockchain, as well as
//! for scanning the Dusk blockchain for requests and for querying
//! contracts' methods.
//!
//! Integration test for the library are provided in the `integration-test`
//! subproject.

mod bc_types;
mod blockchain_payloads;
mod blockchain_queries;
mod circuit;
mod citadel_licenses;
mod citadel_queries;
mod citadel_requests;
mod citadel_types;
mod contract_queries;
mod error;
mod json_loader;
mod utils;

pub mod api;
pub mod license_provider;
pub mod wallet_accessor;

pub use bc_types::*;
pub use blockchain_payloads::{
    PayloadExtractor, PayloadRetriever, PayloadSender,
};
pub use blockchain_queries::{BcInquirer, CrsGetter, TxAwaiter, TxInquirer};
pub use circuit::*;
pub use citadel_licenses::LicenseUser;
pub use citadel_queries::{
    CitadelInquirer, CitadelInquirerWs, LicenseSession, LicenseSessionId,
};
pub use citadel_requests::{RequestCreator, RequestScanner, RequestSender};
pub use citadel_types::*;
pub use contract_queries::{
    block::*, ContractInquirer, ContractInquirerWs, StreamAux,
};
pub use error::Error;
pub use json_loader::JsonLoader;
pub use utils::MoatCoreUtils;
