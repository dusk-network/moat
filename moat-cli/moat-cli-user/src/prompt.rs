// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use requestty::{ErrorKind, Question};

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

pub(crate) fn request_license_hash() -> Result<String, ErrorKind> {
    let q = Question::input("license_hash")
        .message("Please enter license hash:".to_string())
        .validate_on_key(|_, _| true)
        .validate(|license_hash, _| {
            if license_hash.is_empty() {
                Err("Please enter a valid license hash".to_string())
            } else {
                Ok(())
            }
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}

pub(crate) fn request_psk_lp_bytes() -> Result<String, ErrorKind> {
    let q = Question::input("psk_lp_bytes")
        .message("Please enter the address of the LP:".to_string())
        .validate_on_key(|_, _| true)
        .validate(|psk_lp_bytes, _| {
            if psk_lp_bytes.is_empty() {
                Err("Please enter a valid LP address".to_string())
            } else {
                Ok(())
            }
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}

pub(crate) fn request_psk_sp_bytes() -> Result<String, ErrorKind> {
    let q = Question::input("psk_sp_bytes")
        .message("Please enter the address of the SP:".to_string())
        .validate_on_key(|_, _| true)
        .validate(|psk_sp_bytes, _| {
            if psk_sp_bytes.is_empty() {
                Err("Please enter a valid SP address".to_string())
            } else {
                Ok(())
            }
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}

pub(crate) fn request_challenge_bytes() -> Result<String, ErrorKind> {
    let q = Question::input("challenge_bytes")
        .message("Please enter the challenge:".to_string())
        .validate_on_key(|_, _| true)
        .validate(|challenge_bytes, _| {
            if challenge_bytes.is_empty() {
                Err("Please enter a valid challenge".to_string())
            } else {
                Ok(())
            }
        })
        .build();

    let a = requestty::prompt_one(q)?;
    let a_str = a.as_string().expect("answer to be a string").to_string();
    Ok(a_str)
}
