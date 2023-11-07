// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::CliError;
use crate::prompt;
use crate::{Command, Menu};
use dusk_plonk::prelude::{Prover, PublicParameters, Verifier};
use dusk_wallet::WalletPath;
use moat_core::{Error, RequestJson};
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
    IssueLicenseLP,
    ListLicenses,
    UseLicense,
    RequestService,
    GetSession,
    ShowState,
    Exit,
}

fn menu_operation() -> Result<OpSelection, ErrorKind> {
    let cmd_menu = Menu::new()
        .add(CommandMenuItem::SubmitRequest, "Submit Request")
        .add(CommandMenuItem::ListRequestsUser, "List Requests")
        .add(CommandMenuItem::ListRequestsLP, "List Requests (LP)")
        .add(CommandMenuItem::IssueLicenseLP, "Issue License (LP)")
        .add(CommandMenuItem::ListLicenses, "List Licenses")
        .add(CommandMenuItem::UseLicense, "Use License")
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
        CommandMenuItem::SubmitRequest => {
            OpSelection::Run(Box::from(Command::SubmitRequest {
                request_path: prompt::request_pathbuf(
                    "request",
                    "moat-cli/request2.json",
                )?,
            }))
        }
        CommandMenuItem::ListRequestsUser => {
            OpSelection::Run(Box::from(Command::ListRequestsUser))
        }
        CommandMenuItem::ListRequestsLP => {
            OpSelection::Run(Box::from(Command::ListRequestsLP {
                lp_config_path: prompt::request_pathbuf(
                    "LP config",
                    "moat-cli/lp2.json",
                )?,
            }))
        }
        CommandMenuItem::IssueLicenseLP => {
            OpSelection::Run(Box::from(Command::IssueLicenseLP {
                lp_config_path: prompt::request_pathbuf(
                    "LP config",
                    "moat-cli/lp2.json",
                )?,
                request_hash: prompt::request_request_hash()?,
            }))
        }
        CommandMenuItem::ListLicenses => {
            OpSelection::Run(Box::from(Command::ListLicenses {
                request_path: prompt::request_pathbuf(
                    "request",
                    "moat-cli/request2.json",
                )?,
            }))
        }
        CommandMenuItem::UseLicense => {
            OpSelection::Run(Box::from(Command::UseLicense {
                request_path: prompt::request_pathbuf(
                    "request",
                    "moat-cli/request2.json",
                )?,
                license_hash: prompt::request_license_hash()?,
            }))
        }
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
            OpSelection::Run(Box::from(Command::ShowState { dummy: true }))
        }
        CommandMenuItem::Exit => OpSelection::Exit,
    })
}

pub struct SetupHolder {
    pub pp: PublicParameters,
    pub prover: Prover,
    pub verifier: Verifier,
}

pub struct Interactor {
    pub wallet_path: WalletPath,
    pub psw: Password,
    pub blockchain_access_config: BlockchainAccessConfig,
    pub lp_config_path: PathBuf,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub request_json: Option<RequestJson>,
    pub setup_holder: Option<SetupHolder>,
}

impl Interactor {
    pub async fn run_loop(&mut self) -> Result<(), CliError> {
        loop {
            let op = menu_operation()?;
            match op {
                OpSelection::Exit => return Ok(()),
                OpSelection::Run(command) => {
                    let result = command
                        .run(
                            &self.wallet_path,
                            &self.psw,
                            &self.blockchain_access_config,
                            &self.lp_config_path,
                            self.gas_limit,
                            self.gas_price,
                            self.request_json.clone(),
                            &mut self.setup_holder,
                        )
                        .await;
                    if result.is_err() {
                        let error = result.unwrap_err();
                        match error {
                            Error::IO(arc) => {
                                println!("{}", arc.as_ref().to_string());
                            }
                            Error::Transaction(bx) => {
                                println!("{}", bx.as_ref().to_string());
                            }
                            _ => {
                                println!("{:?}", error);
                            }
                        }
                    }
                    continue;
                }
            }
        }
    }
}
