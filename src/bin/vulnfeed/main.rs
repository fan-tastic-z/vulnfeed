use clap::Parser;
use vulnfeed::{
    AppResult, cli,
    utils::{styled::styled, version::version},
};

#[derive(Debug, clap::Parser)]
#[command(name = "vulnfeed", version, long_version = version(), styles=styled())]
struct Command {
    #[clap(subcommand)]
    cmd: SubCommand,
}

impl Command {
    pub fn run(self) -> AppResult<()> {
        match self.cmd {
            SubCommand::Server(cmd) => cmd.run(),
            SubCommand::CreateSuperUser(cmd) => cmd.run(),
        }
    }
}

#[derive(Debug, clap::Subcommand)]
enum SubCommand {
    Server(cli::CommandStart),
    CreateSuperUser(cli::CreateSuperUser),
}

fn main() -> AppResult<()> {
    let cmd = Command::parse();
    cmd.run()
}
