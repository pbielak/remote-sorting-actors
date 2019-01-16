/*
Command line args used by supervisor and sorting actor
*/
use std::path::PathBuf;


#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct SupervisorCliArgs {
    #[structopt(short = "i", long = "input", parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    pub output: PathBuf,

    #[structopt(short = "n", long = "nb-actors")]
    pub n: usize,

    #[structopt(short = "k", long = "nb-chunks")]
    pub k: usize,

    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    #[structopt(long = "addr")]
    pub addr: String,
}


#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct SortingActorCliArgs {
    #[structopt(long = "supervisor-addr")]
    pub supervisor_addr: String,

    #[structopt(short = "d", long = "debug")]
    pub debug: bool,
}
