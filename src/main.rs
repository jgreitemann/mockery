mod app;
mod ast_iterators;
mod cli;
mod error;
mod fs_iterators;
mod mock_generation;

#[cfg(test)]
mod test_utils;

use crate::app::*;
use crate::cli::*;
use crate::error::*;
use clang::*;
use clap::Parser;

fn main() {
    let res = cli_main();
    if res.is_err() {
        std::process::exit(res.report());
    }
}

fn cli_main() -> CLIResult<()> {
    let opts = MockeryOpts::parse();

    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, true);

    let app = MockeryApp::new(&index, &opts)?;

    match opts.subcmd {
        SubCommand::Create(crt) => app.run_create(crt),
        SubCommand::Update(upd) => app.run_update(upd),
        SubCommand::Dump(dmp) => app.run_dump(dmp),
    }
}
