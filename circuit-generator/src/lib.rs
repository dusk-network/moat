// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod cdef_parser;
mod error;
mod types;

pub use cdef_parser::parse_cdef;
pub use error::Error;
pub use types::UserAttributes;
