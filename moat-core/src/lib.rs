// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

//! Library for submitting license requests to the Dusk blockchain, as well as
//! for scanning the Dusk blockchain for requests.
//!
//! Object `RequestSender` provides methods `send` for sending requests to the
//! blockchain.
//!
//! Object `RequestScanner` provides methods `scan_last_blocks` and
//! `scan_block_range` for scanning blocks for requests. A simple, efficient
//! loop can be programmed to scan the entire blockchain.
//!
//! Other objects are provided as helpers and more generic utilities:
//!
//! `RequestCreator` can be used to create an license request object from
//! scratch.
//!
//! `TxsRetriever` retrieves transactions contained within a block or a block
//! range.
//!
//! `PayloadRetriever` allows for retrieving payload of any type from a
//! transaction with a given  transaction id.
//!
//! `RequestExtractor` extracts license request from a given transaction object.
//!
//! `RequestJson` allows for storage of initial data from which a license
//! request can be created.
//!
//! `JsonLoader` is an utility trait which allows for seamless enrichment of any
//! structure with an ability to be loaded from a disk file in a json format.
//!
//! `retrieval_types` contains a set of types used when retrieving blockchain
//! data, from transactions and headers to blocks and contract-call-info.
//!
//! The library has been architected in such a way so that it should be very
//! easy to extend it to get support for a different type of payload or a
//! different type or specifics of information extracted from blockchain.
//!
//! Integration test for the library are provided in the `integration-test`
//! subproject.

mod blockchain_requests;
mod error;
mod json_loader;
mod tx_retrievals;
mod types;

pub use blockchain_requests::{
    PayloadExtractor, PayloadRetriever, PayloadSender, RequestCreator,
    RequestScanner,
};
pub use error::Error;
pub use json_loader::JsonLoader;
pub use tx_retrievals::TxRetriever;
pub use types::*;
