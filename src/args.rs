/*
Define input args for application
*/

use std::path::PathBuf;


#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct CliArgs {
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
}


