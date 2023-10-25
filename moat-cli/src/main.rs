// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

mod args;
mod interactive;
mod menu;
mod command;

use crate::args::Args;
use crate::menu::Menu;
use crate::command::Command;

use clap::Parser;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();
    let _json_path = cli.json_path.as_path();
    interactive::run_loop().await?;

    Ok(())
}
