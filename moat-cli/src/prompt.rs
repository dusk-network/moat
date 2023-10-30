// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use requestty::{ErrorKind, Question};

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
