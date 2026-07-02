mod executor;
use anyhow::Context;
use constants::{
    CONFIG_PATH, PROMPT_DEFAULT, PROMPT_SECTION, PROMPT_SECTION_PROMPT,
    PROMPT_SECTION_PROMPT_COLOR_KEY, WELCOME_MESSAGE,
};
use engine::{lexer::Lexer, parser::Parser, readline::read_line, state::ShellState};
use errors::errors::{ShellErrorResault, ShellErrors};
use helpers::io::expand_path;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{
    io::{AsyncWriteExt, BufWriter},
    task::JoinHandle,
};

use crate::executor::execute;

fn create_default_config() -> anyhow::Result<()> {
    let config_path = expand_path(CONFIG_PATH);

    if config_path.exists() {
        return Ok(());
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let contents = format!(
        "[{PROMPT_SECTION}]\n\
         {PROMPT_SECTION_PROMPT} = \"{PROMPT_DEFAULT}\"\n\
         {PROMPT_SECTION_PROMPT_COLOR_KEY} = \"White\"\n"
    );

    fs::write(&config_path, contents)
        .with_context(|| format!("failed to write {}", config_path.display()))?;

    Ok(())
}

fn spawn_command_handler(state: ShellState) -> JoinHandle<ShellErrorResault<()>> {
    tokio::spawn(async move {
        let state = Arc::new(Mutex::new(state));
        let stdout = tokio::io::stdout();
        let mut stdout = BufWriter::new(stdout);

        stdout
            .write(format!("{}\n", WELCOME_MESSAGE).as_bytes())
            .await?;
        stdout.flush().await?;

        loop {
            let state_clone = Arc::clone(&state);
            let state_for_read = Arc::clone(&state_clone);
            let line = tokio::task::spawn_blocking(move || {
                let mut s = state_for_read.blocking_lock();
                read_line(&mut s)
            })
            .await?;

            let line = match line {
                Some(l) => l,
                None => break,
            };

            let mut lexer = Lexer::new(line.clone());
            let tokens = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            let shell = parser.parse();
            let mut state = state_clone.lock().await;
            if execute(shell, &mut *state).await.is_err() {
                stdout
                    .write(format!("{}\n", ShellErrors::CommandNotFound(line.clone())).as_bytes())
                    .await?;
                stdout.flush().await?;
            }
            stdout.write_all(b"\n").await?;
            stdout.flush().await?;
        }

        Ok(())
    })
}

#[tokio::main]
async fn main() -> Result<(), ShellErrors> {
    create_default_config().ok();
    let state = ShellState::new();
    let command_handler = spawn_command_handler(state);
    if let Ok(Err(e)) = command_handler.await {
        eprintln!("{}", ShellErrors::CommandNotFound(e.to_string()));
    }
    unreachable!("Main, This code should not be reached, as it comes after the REPL loop");
}
