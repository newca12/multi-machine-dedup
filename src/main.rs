use clap::Parser;
use env_logger;
use multi_machine_dedup::SubCommand;
//use structopt::StructOpt;
fn main() {
    env_logger::init();
    let args = multi_machine_dedup::CLI::parse();
    match args.cmd {
        SubCommand::Index(args) => multi_machine_dedup::index(args),
        SubCommand::CheckIntegrity(args) => multi_machine_dedup::check_integrity(args),
    }
}
