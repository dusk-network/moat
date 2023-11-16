// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::error::CliError;
use crate::prompt;
use crate::{Command, Menu};
use dusk_pki::SecretSpendKey;
use dusk_plonk::prelude::{Prover, PublicParameters, Verifier};
use dusk_wallet::WalletPath;
use moat_core::Error;
use requestty::{ErrorKind, Question};
use wallet_accessor::{BlockchainAccessConfig, Password};

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

pub struct SetupHolder {
    pub pp: PublicParameters,
    pub prover: Prover,
    pub verifier: Verifier,
}

pub struct Interactor {
    pub wallet_path: WalletPath,
    pub psw: Password,
    pub blockchain_access_config: BlockchainAccessConfig,
    pub gas_limit: u64,
    pub gas_price: u64,
    pub ssk: SecretSpendKey,
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
                            self.gas_limit,
                            self.gas_price,
                            self.ssk,
                            &mut self.setup_holder,
                        )
                        .await;
                    match result {
                        Ok(run_result) => {
                            println!("{}", run_result);
                        }
                        Err(error) => {
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
                            println!();
                        }
                    }
                    continue;
                }
            }
        }
    }
}
