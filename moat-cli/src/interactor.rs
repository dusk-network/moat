// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::CliError;
use crate::prompt;
use crate::{Command, Menu};
use dusk_wallet::WalletPath;
use moat_core::RequestJson;
use requestty::{ErrorKind, Question};
use std::path::PathBuf;
use wallet_accessor::{BlockchainAccessConfig, Password};

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
enum OpSelection {
    Run(Box<Command>),
    Exit,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum CommandMenuItem {
    SubmitRequest,
    ListRequestsUser,
    ListRequestsLP,
    Exit,
}

fn menu_operation() -> Result<OpSelection, ErrorKind> {
    let cmd_menu = Menu::new()
        .add(CommandMenuItem::SubmitRequest, "Submit Request")
        .add(CommandMenuItem::ListRequestsUser, "List Requests (User)")
        .add(CommandMenuItem::ListRequestsLP, "List Requests (LP)")
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
                provider_psk: prompt::request_provider_psk()?,
            }))
        }
        CommandMenuItem::ListRequestsUser => {
            OpSelection::Run(Box::from(Command::ListRequestsUser {
                dummy: true,
            }))
        }
        CommandMenuItem::ListRequestsLP => {
            OpSelection::Run(Box::from(Command::ListRequestsLP { dummy: true }))
        }
        CommandMenuItem::Exit => OpSelection::Exit,
    })
}

pub struct Interactor {
    pub wallet_path: WalletPath,
    pub psw: Password,
    pub blockchain_access_config: BlockchainAccessConfig,
    pub lp_config_path: PathBuf,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub request_json: Option<RequestJson>,
}

impl Interactor {
    pub async fn run_loop(&self) -> Result<(), CliError> {
        loop {
            let op = menu_operation()?;
            match op {
                OpSelection::Exit => return Ok(()),
                OpSelection::Run(command) => {
                    command
                        .run(
                            &self.wallet_path,
                            &self.psw,
                            &self.blockchain_access_config,
                            &self.lp_config_path,
                            self.gas_limit,
                            self.gas_price,
                            self.request_json.clone(),
                        )
                        .await?
                }
            }
        }
    }
}
