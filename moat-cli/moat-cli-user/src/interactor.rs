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
    SubmitRequest,
    ListLicenses,
    UseLicense,
    RequestService,
    ShowState,
    Exit,
}

fn menu_operation() -> Result<OpSelection, ErrorKind> {
    let cmd_menu = Menu::new()
        .add(CommandMenuItem::SubmitRequest, "Submit Request")
        .add(CommandMenuItem::ListLicenses, "List Licenses")
        .add(CommandMenuItem::UseLicense, "Use License")
        .add(
            CommandMenuItem::RequestService,
            "Request Service (Off-Chain)",
        )
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
        CommandMenuItem::SubmitRequest => {
            OpSelection::Run(Box::from(Command::SubmitRequest {
                psk_lp_bytes: prompt::request_psk_lp_bytes()?,
            }))
        }
        CommandMenuItem::ListLicenses => {
            OpSelection::Run(Box::from(Command::ListLicenses))
        }
        CommandMenuItem::UseLicense => {
            OpSelection::Run(Box::from(Command::UseLicense {
                license_hash: prompt::request_license_hash()?,
                psk_lp_bytes: prompt::request_psk_lp_bytes()?,
                psk_sp_bytes: prompt::request_psk_sp_bytes()?,
                challenge_bytes: prompt::request_challenge_bytes()?,
            }))
        }
        CommandMenuItem::RequestService => {
            OpSelection::Run(Box::from(Command::RequestService {
                session_cookie: prompt::request_session_cookie()?,
            }))
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
