use clap::Parser;
use vulnfeed::{cli, errors::Error, utils::{styled::styled, version::version}};
use error_stack::Result;



#[derive(Debug, clap::Parser)]
#[command(name = "vulnfeed", version, long_version = version(), styles=styled())]
struct Command {
    #[clap(subcommand)]
    cmd: SubCommand,
}

impl Command {
    pub fn run(self) -> Result<(), Error> {
        match self.cmd {
            SubCommand::Server(cmd) => cmd.run(),
        }
    }
}

#[derive(Debug, clap::Subcommand)]
enum SubCommand {
    Server(cli::CommandStart),
}



fn main() -> Result<(), Error> {
    let cmd = Command::parse();
    cmd.run()
}