mod app;
mod ast_iterators;
mod cli;
mod fs_iterators;
mod mock_generation;

#[cfg(test)]
mod test_utils;

use crate::app::*;
use crate::cli::*;
use clang::*;
use clap::Parser;

fn main() {
    match cli_main() {
        Ok(()) => (),
        Err(CLIError(msg)) => {
            eprintln!("error: {}", msg);
            std::process::exit(1);
        }
    }
}

fn cli_main() -> CLIResult {
    let opts = MockeryOpts::parse();

    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, true);

    let app = MockeryApp::new(&index, &opts);

    match opts.subcmd {
        SubCommand::Create(crt) => app.run_create(crt),
        SubCommand::Update(upd) => app.run_update(upd),
        SubCommand::Dump(dmp) => app.run_dump(dmp),
    }
}
