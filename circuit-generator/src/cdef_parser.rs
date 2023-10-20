// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Error, UserAttributes};
use crate::Error::Parsing;
use pest::Parser;
use pest_derive::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[grammar = "cdef.pest"]
pub struct CDefParser;

pub fn parse_cdef(path: impl AsRef<Path>) -> Result<UserAttributes, Error> {
    let unparsed_file = fs::read_to_string(path)?;
    let file = CDefParser::parse(Rule::file, &unparsed_file)
        .map_err(|e| Parsing(e.variant.message().into_owned()))?
        .next()
        .unwrap();

    let mut user_attributes = UserAttributes { country_code: None, age: None };

    for property in file.into_inner() {
        match property.as_rule() {
            Rule::property => {
                let mut inner_rules = property.into_inner(); // { name ~ ":" ~ value }
                let name = inner_rules.next().unwrap().as_str();
                let value = inner_rules.next().unwrap().as_str();
                match name {
                    "country" => user_attributes.country_code = Some(value.parse::<u16>().unwrap()),
                    "age" => user_attributes.age = Some(value.parse::<u8>().unwrap()),
                    _ => (),
                }
            }
            Rule::EOI => (),
                _ => unreachable!(),
        }
    }

    Ok(user_attributes)
}
