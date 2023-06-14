// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::RequestJson;
use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct Request(pub Vec<u8>);

#[derive(Debug, Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct RequestBody {
    pub user: String,
    pub provider: String,
    pub body: String,
}

impl Request {
    pub fn from(request_json: &RequestJson) -> Request {
        let rb = RequestBody {
            user: request_json.user.clone(),
            provider: request_json.provider.clone(),
            body: request_json.body.clone(),
        };
        let serialized = rkyv::to_bytes::<_, 16384>(&rb)
            .expect("Request serialization should work")
            .to_vec();
        Request(serialized)
    }
}

impl RequestBody {
    pub fn from(buf: Vec<u8>) -> RequestBody {
        let archived =
            unsafe { rkyv::archived_root::<RequestBody>(buf.as_slice()) };
        let deserialized: RequestBody =
            archived.deserialize(&mut rkyv::Infallible).unwrap();
        return deserialized;
    }
}
