use clap::Parser;
use env_logger::{self, Env};
use multi_machine_dedup::Commands;

fn main() {
    env_logger::Builder::from_env(Env::default().filter_or("LOG", "info"))
        .format_timestamp(None)
        .init();
    let args = multi_machine_dedup::CLI::parse();
    match args.cmd {
        Commands::Index(args) => multi_machine_dedup::index(args),
        Commands::CheckIntegrity(args) => multi_machine_dedup::check_integrity(args),
        Commands::Compare(args) => multi_machine_dedup::compare(args),
    }
}
