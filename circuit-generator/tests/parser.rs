// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use circuit_generator::{parse_cdef, Error, UserAttributes};

fn assert_user_attributes(user_attributes: &UserAttributes) {
    assert_eq!(user_attributes.country_code, Some(48));
    assert_eq!(user_attributes.age, Some(21));
}

#[test]
fn parse_country_age_valid() -> Result<(), Error> {
    let path_valid1 = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/cdefs/country_age_valid1.cdef"
    );
    let path_valid2 = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/cdefs/country_age_valid2.cdef"
    );
    assert_user_attributes(&parse_cdef(path_valid1)?);
    assert_user_attributes(&parse_cdef(path_valid2)?);
    Ok(())
}

#[test]
fn parse_country_age_not_valid() -> Result<(), Error> {
    let path_not_valid = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/cdefs/country_age_not_valid.cdef"
    );
    assert!(parse_cdef(path_not_valid).is_err());
    Ok(())
}
