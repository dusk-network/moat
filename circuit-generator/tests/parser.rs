// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use circuit_generator::{parse_cdef, Error};

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
    parse_cdef(path_valid1)?;
    parse_cdef(path_valid2)?;
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
