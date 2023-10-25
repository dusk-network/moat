// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::{Command, Menu};
use dusk_wallet::WalletPath;
use moat_core::RequestJson;
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
    ListRequests,
    Exit,
}

fn menu_operation() -> Result<OpSelection, ErrorKind> {
    let cmd_menu = Menu::new()
        .add(CommandMenuItem::SubmitRequest, "Submit Request")
        .add(CommandMenuItem::ListRequests, "List Requests")
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
            OpSelection::Run(Box::from(Command::SubmitRequest { dummy: true }))
        }
        CommandMenuItem::ListRequests => {
            OpSelection::Run(Box::from(Command::ListRequests { dummy: true }))
        }
        CommandMenuItem::Exit => OpSelection::Exit,
    })
}

pub async fn run_loop(
    wallet_path: &WalletPath,
    psw: &Password,
    blockchain_access_config: &BlockchainAccessConfig,
    gas_limit: u64,
    gas_price: u64,
    request_json: Option<RequestJson>,
) -> Result<(), moat_core::Error> {
    // todo: error type
    loop {
        let request_json = request_json.clone(); // todo: introduce object with state here
        let op = menu_operation().map_err(|_| moat_core::Error::Rkyv)?; // todo: change the bogus error here
        match op {
            OpSelection::Exit => return Ok(()),
            OpSelection::Run(bx) => {
                bx.run(
                    wallet_path,
                    psw,
                    blockchain_access_config,
                    gas_limit.clone(),
                    gas_price.clone(),
                    request_json,
                )
                .await?
            }
        }
    }
}
