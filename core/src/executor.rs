use commands::execute_builtin;
use engine::{
    ast::{CommandType, Shell},
    state::ShellState,
};

pub async fn execute(
    shell: Shell,
    state: &mut ShellState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for pipeline in shell.pipelines {
        for command in pipeline.commands {
            match command.command_type {
                CommandType::Builtin(cmd) => {
                    execute_builtin(cmd, state)?;
                }

                CommandType::External { program, args } => {
                    tokio::process::Command::new(program)
                        .args(args)
                        .spawn()?
                        .wait()
                        .await?;
                }
            }
        }
    }

    Ok(())
}
