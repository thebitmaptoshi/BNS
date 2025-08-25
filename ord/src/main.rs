use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    // Existing subcommands
    Server,
    Wallet,
    #[cfg(feature = "bitmap")]
    Bitmap,
}

fn main() {
    let args = Args::parse();
    let config = crate::config::Config::load_config("~/.ord/config.toml");
    match args.command {
        Command::Server => { /* Existing logic */ },
        Command::Wallet => { /* Existing logic */ },
        #[cfg(feature = "bitmap")]
        Command::Bitmap => crate::subcommand::bitmap::run(config),
    }
}
