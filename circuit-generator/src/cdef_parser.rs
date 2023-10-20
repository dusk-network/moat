// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::Error;
use crate::Error::Parsing;
use pest::Parser;
use pest_derive::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "cdef.pest"]
pub struct CDefParser;

pub fn parse_cdef(path: impl AsRef<Path>) -> Result<(), Error> {
    let unparsed_file = fs::read_to_string(path)?;
    let _file = CDefParser::parse(Rule::file, &unparsed_file)
        .map_err(|e| Parsing(e.variant.message().into_owned()))?
        .next()
        .unwrap();
    Ok(())

    // for property in file.into_inner() {
    //     match property.as_rule() {
    //         Rule::property => {
    //             println!("{}", property);
    //         }
    //         Rule::EOI => (),
    //             _ => unreachable!(),
    //     }
    // }
}
