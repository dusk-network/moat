// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct RequestJson {
    pub user_ssk: String,
    pub provider_psk: String,
}

impl RequestJson {
    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<RequestJson, Error> {
        let mut content = String::new();
        let file = File::open(path.as_ref())?;
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut content)?;
        serde_json::from_str(&content).map_err(|e| e.into())
    }
}
