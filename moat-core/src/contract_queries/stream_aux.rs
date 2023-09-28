// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::contract_queries::block::BlockInPlace;
use crate::Error;
use bytecheck::CheckBytes;
use bytes::Bytes;
use futures_util::StreamExt;
use rkyv::de::deserializers::SharedDeserializeMap;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{Archive, Deserialize, Infallible};

pub struct StreamAux;

impl StreamAux {
    /// Finds and returns first item for which
    /// the given filter returns true,
    /// returns error if no such item was found
    pub fn find_item<R, const L: usize>(
        filter: impl Fn(&R) -> Result<bool, Error>,
        mut stream: impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>
            + std::marker::Unpin,
    ) -> Result<R, Error>
    where
        R: Archive,
        R::Archived: Deserialize<R, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>
            + Deserialize<R, SharedDeserializeMap>,
    {
        let mut buffer = vec![];
        while let Some(http_chunk) = stream.next().wait() {
            buffer.extend_from_slice(
                &http_chunk
                    .map_err(|_| Error::Stream(Box::from("chunking error")))?,
            );
            let mut chunk = buffer.chunks_exact(L);
            for bytes in chunk.by_ref() {
                let item: R = rkyv::from_bytes(bytes).map_err(|_| {
                    Error::Stream(Box::from("deserialization error"))
                })?;
                if filter(&item)? {
                    return Ok(item);
                }
            }
            buffer = chunk.remainder().to_vec();
        }
        Err(Error::Stream(Box::from("item not found")))
    }

    /// Collects all items and returns them in a vector,
    /// returns empty vector if no items were present.
    pub fn collect_all<R, const L: usize>(
        mut stream: impl futures_core::Stream<Item = Result<Bytes, reqwest::Error>>
            + std::marker::Unpin,
    ) -> Result<Vec<R>, Error>
    where
        R: Archive,
        R::Archived: Deserialize<R, Infallible>
            + for<'b> CheckBytes<DefaultValidator<'b>>
            + Deserialize<R, SharedDeserializeMap>,
    {
        let mut vec = vec![];
        let mut buffer = vec![];
        while let Some(http_chunk) = stream.next().wait() {
            buffer.extend_from_slice(
                &http_chunk
                    .map_err(|_| Error::Stream(Box::from("chunking error")))?,
            );
            let mut chunk = buffer.chunks_exact(L);
            for bytes in chunk.by_ref() {
                let item: R = rkyv::from_bytes(bytes).map_err(|_| {
                    Error::Stream(Box::from("deserialization error"))
                })?;
                vec.push(item);
            }
            buffer = chunk.remainder().to_vec();
        }
        Ok(vec)
    }
}
