// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

// use rkyv::{
//     archived_root, Archive, Archived, Deserialize, Infallible, Serialize,
// };

use crate::error::Error;
use crate::retrieval_types::Blocks;
use gql_client::Client;
use wallet_accessor::BlockchainAccessConfig;

pub struct RequestRetriever;

impl RequestRetriever {
    pub async fn retrieve_block(
        cfg: &BlockchainAccessConfig,
    ) -> Result<(), Error> {
        let client = Client::new(cfg.graphql_address.clone());

        let block_height = "97117";

        // todo: this will fail if there are no transactions in a given block,
        // so first we need to make sure that a block has transactions
        let query =
            "{blocks(height:9999){ header{height, seed }, transactions{txid, json}}}".replace("9999", block_height);

        let response = client.query::<Blocks>(&query).await.expect("todo:"); // todo: remove expect and replace with ?

        println!("resp={:?}", response);
        println!("blocks={:?}", response.unwrap());

        Ok(())
    }
}
