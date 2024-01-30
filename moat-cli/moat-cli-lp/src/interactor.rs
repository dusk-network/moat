// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::prompt;
use crate::{Command, Menu};
use moat_cli_common::Error;
use requestty::{ErrorKind, Question};
use zk_citadel_moat::api::MoatContext;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
enum OpSelection {
    Run(Box<Command>),
    Exit,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum CommandMenuItem {
    ListRequestsLP,
    IssueLicenseLP,
    ListLicenses,
    ShowState,
    Exit,
}

fn menu_operation() -> Result<OpSelection, ErrorKind> {
    let cmd_menu = Menu::new()
        .add(CommandMenuItem::ListRequestsLP, "List Requests")
        .add(CommandMenuItem::IssueLicenseLP, "Issue License")
        .add(CommandMenuItem::ListLicenses, "List Licenses")
        .add(CommandMenuItem::ShowState, "Show state")
        .separator()
        .add(CommandMenuItem::Exit, "Exit");

    let q = Question::select("theme")
        .message("What would you like to do?")
        .choices(cmd_menu.clone())
        .build();

    let answer = requestty::prompt_one(q)?;
    let cmd = cmd_menu.answer(&answer).to_owned();
    Ok(match cmd {
        CommandMenuItem::ListRequestsLP => {
            OpSelection::Run(Box::from(Command::ListRequestsLP))
        }
        CommandMenuItem::IssueLicenseLP => {
            OpSelection::Run(Box::from(Command::IssueLicenseLP {
                request_hash: prompt::request_request_hash()?,
                attr_data_bytes: prompt::request_attr_data()?,
            }))
        }
        CommandMenuItem::ListLicenses => {
            OpSelection::Run(Box::from(Command::ListLicenses))
        }
        CommandMenuItem::ShowState => {
            OpSelection::Run(Box::from(Command::ShowState))
        }
        CommandMenuItem::Exit => OpSelection::Exit,
    })
}

pub struct Interactor {
    pub moat_context: MoatContext,
}

impl Interactor {
    pub async fn run_loop(&mut self) -> Result<(), Error> {
        loop {
            let op = menu_operation()?;
            match op {
                OpSelection::Exit => return Ok(()),
                OpSelection::Run(command) => {
                    let result = command.run(&self.moat_context).await;
                    match result {
                        Ok(run_result) => {
                            println!("{}", run_result);
                        }
                        Err(error) => {
                            println!("{}", error);
                        }
                    }
                    continue;
                }
            }
        }
    }
}
