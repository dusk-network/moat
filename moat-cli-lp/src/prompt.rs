// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use requestty::{ErrorKind, Question};

pub(crate) fn request_request_hash() -> Result<String, ErrorKind> {
    let q = Question::input("request_hash")
        .message("Please enter request hash:".to_string())
        .validate_on_key(|_, _| {
            true // todo: add some validation of the request hash
        })
        .validate(|request_hash, _| {
            if request_hash.is_empty() {
                Err("Please enter a valid request hash".to_string())
            } else {
                Ok(())
            }
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}

pub(crate) fn request_attr_data() -> Result<String, ErrorKind> {
    let q = Question::input("attr_data")
        .message("Please enter the attribute data:".to_string())
        .validate_on_key(|_, _| {
            true // todo: add some validation of the attr_data
        })
        .validate(|attr_data, _| {
            if attr_data.is_empty() {
                Err("Please enter valid attribute data".to_string())
            } else {
                Ok(())
            }
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}
