// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use crate::Menu;
use requestty::{ErrorKind, Question};

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum CommandMenuItem {
    SubmitRequest,
    ListRequests,
    Exit,
}

fn menu_operation() -> Result<(), ErrorKind> {
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
    match cmd {
        CommandMenuItem::SubmitRequest => println!("do submit request"),
        CommandMenuItem::ListRequests => println!("do list requests"),
        CommandMenuItem::Exit => {
            println!("do exit, bye bye");
            return Err(ErrorKind::Aborted);
        }
    };
    Ok(())
}

pub async fn run_loop() -> Result<(), ErrorKind> {
    loop {
        let op = menu_operation();
        if op.is_err() {
            return Ok(());
        }
    }
}
