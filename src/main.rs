use multi_machine_dedup::SubCommand;
use structopt::StructOpt;
fn main() {
    let args = multi_machine_dedup::CLI::from_args();
    match args.cmd {
        SubCommand::Index(args) => multi_machine_dedup::index(args),
        SubCommand::CheckIntegrity(args) => multi_machine_dedup::check_integrity(args),
    }
}
