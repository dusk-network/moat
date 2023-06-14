// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use rkyv::{Archive, Deserialize};

use crate::error::Error;
use crate::request::Request;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct RequestJson {
    pub user: String,
    pub provider: String,
    pub body: String,
}

#[derive(Debug, Archive, rkyv::Deserialize, rkyv::Serialize)]
pub struct RequestBody {
    pub user: String,
    pub provider: String,
    pub body: String,
}

impl RequestJson {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<RequestJson, Error> {
        let mut content = String::new();
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut content)?;
        serde_json::from_str(&content).map_err(|e| e.into())
    }

    pub fn to_request(&self) -> Request {
        let serialized = serde_json::to_string(&self)
            .expect("Request json serialization should work");
        Request(serialized.as_bytes().to_vec())
    }

    pub fn to_request_rkyv(&self) -> Request {
        let rb = RequestBody {
            user: self.user.clone(),
            provider: self.provider.clone(),
            body: self.body.clone(),
        };
        let serialized = rkyv::to_bytes::<_, 16384>(&rb)
            .expect("Request serialization should work").to_vec();
        Request(serialized)
    }

    pub fn from_request_rkyv(buf: Vec<u8>) -> RequestBody {
        let archived = unsafe { rkyv::archived_root::<RequestBody>(buf.as_slice()) };
        let deserialized: RequestBody = archived.deserialize(&mut rkyv::Infallible).unwrap();
        return deserialized
    }
}
