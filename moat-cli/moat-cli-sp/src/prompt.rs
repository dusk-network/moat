// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use requestty::{ErrorKind, Question};

pub(crate) fn request_session_id() -> Result<String, ErrorKind> {
    let q = Question::input("session_id")
        .message("Please enter session id:".to_string())
        .validate_on_key(|_, _| true)
        .validate(|id, _| {
            if id.is_empty() {
                Err("Please enter a valid session id".to_string())
            } else {
                Ok(())
            }
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}

pub(crate) fn request_session_cookie() -> Result<String, ErrorKind> {
    let q = Question::input("session_cookie")
        .message("Please enter session cookie:".to_string())
        .validate_on_key(|_, _| true)
        .validate(|id, _| {
            if id.is_empty() {
                Err("Please enter a valid session cookie".to_string())
            } else {
                Ok(())
            }
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}

pub(crate) fn request_psk_lp() -> Result<String, ErrorKind> {
    let q = Question::input("psk_lp_bytes")
        .message("Please enter the LP address:".to_string())
        .validate_on_key(|_, _| true)
        .validate(|id, _| {
            if id.is_empty() {
                Err("Please enter a valid address for the LP".to_string())
            } else {
                Ok(())
            }
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}
