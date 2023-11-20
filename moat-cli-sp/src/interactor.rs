// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::config::SPCliConfig;
use crate::prompt;
use crate::{Command, Menu};
use moat_cli_common::Error;
use dusk_wallet::WalletPath;
use requestty::{ErrorKind, Question};
use wallet_accessor::{BlockchainAccessConfig, Password};

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
enum OpSelection {
    Run(Box<Command>),
    Exit,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum CommandMenuItem {
    RequestService,
    GetSession,
    ShowState,
    Exit,
}

fn menu_operation() -> Result<OpSelection, ErrorKind> {
    let cmd_menu = Menu::new()
        .add(
            CommandMenuItem::RequestService,
            "Request Service (Off-Chain)",
        )
        .add(CommandMenuItem::GetSession, "Get Session (SP)")
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
        CommandMenuItem::RequestService => {
            OpSelection::Run(Box::from(Command::RequestService {
                session_cookie: prompt::request_session_cookie()?,
            }))
        }
        CommandMenuItem::GetSession => {
            OpSelection::Run(Box::from(Command::GetSession {
                session_id: prompt::request_session_id()?,
            }))
        }
        CommandMenuItem::ShowState => {
            OpSelection::Run(Box::from(Command::ShowState))
        }
        CommandMenuItem::Exit => OpSelection::Exit,
    })
}

pub struct Interactor {
    pub wallet_path: WalletPath,
    pub psw: Password,
    pub blockchain_access_config: BlockchainAccessConfig,
    pub config: SPCliConfig,
    pub gas_limit: u64,
    pub gas_price: u64,
}

impl Interactor {
    pub async fn run_loop(&mut self) -> Result<(), Error> {
        loop {
            let op = menu_operation()?;
            match op {
                OpSelection::Exit => return Ok(()),
                OpSelection::Run(command) => {
                    let result = command
                        .run(&self.blockchain_access_config, &self.config)
                        .await;
                    match result {
                        Ok(run_result) => {
                            println!("{}", run_result);
                        }
                        Err(error) => {
                            println!("{}", error.to_string());
                        }
                    }
                    continue;
                }
            }
        }
    }
}
