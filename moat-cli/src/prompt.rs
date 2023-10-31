// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use requestty::{ErrorKind, Question};
use std::path::PathBuf;

pub(crate) fn request_provider_psk() -> Result<String, ErrorKind> {
    let q = Question::input("psk")
        .message("Please enter the provider public spend key:".to_string())
        .validate_on_key(|_, _| {
            true // todo: add some validation of the psk
        })
        .validate(|_, _| {
            Ok(()) // todo: add some validation of the psk
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}

pub(crate) fn request_session_id() -> Result<String, ErrorKind> {
    let q = Question::input("session_id")
        .message("Please enter session id:".to_string())
        .validate_on_key(|_, _| {
            true // todo: add some validation of the session id
        })
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

pub(crate) fn request_pathbuf(
    hint: &str,
) -> Result<Option<PathBuf>, ErrorKind> {
    let q = Question::input("psk")
        .message(format!("Please enter path for {}:", hint))
        .validate_on_key(|_, _| {
            true // todo: add some validation of the psk
        })
        .validate(|_, _| {
            Ok(()) // todo: add some validation of the psk
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(if a_str.is_empty() {
        None
    } else {
        Some(PathBuf::from(a_str))
    })
}
