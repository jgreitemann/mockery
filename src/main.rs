mod ast_iterators;
mod cli;
mod fs_iterators;
mod mock_generation;

#[cfg(test)]
mod test_utils;

use crate::ast_iterators::print_ast;
use crate::cli::*;
use crate::fs_iterators::*;
use crate::mock_generation::*;
use clang::*;
use clap::Parser;
use std::path::{Path, PathBuf};

fn find_compilation_database(starting_point: &Path, radius: usize) -> Result<PathBuf, ()> {
    FilesystemDirectoryNode {
        path: std::fs::canonicalize(starting_point).map_err(|_| ())?,
    }
    .search(radius)
    .find(|path| path.join("compile_commands.json").exists())
    .ok_or(())
}

fn main() -> Result<(), String> {
    let opts = MockeryOpts::parse();

    let source_file = std::fs::canonicalize(match &opts.subcmd {
        SubCommand::Create(crt) => &crt.interface_source[..],
        SubCommand::Update(upd) => &upd.mock_source[..],
        SubCommand::Dump(dmp) => &dmp.source[..],
    })
    .unwrap();

    let compile_db_dir = opts
        .compile_commands
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            find_compilation_database(source_file.parent().unwrap(), opts.search_radius).unwrap()
        });

    std::env::set_current_dir(&compile_db_dir).unwrap();

    let compile_db = CompilationDatabase::from_directory(compile_db_dir).unwrap();

    let commands = compile_db.get_compile_commands(source_file).unwrap();
    let command_vec = commands.get_commands();
    let command = command_vec.first().unwrap();
    let filename = command.get_filename();
    let args: Vec<_> = command
        .get_arguments()
        .into_iter()
        .filter(|a| PathBuf::from(a) != filename)
        .collect();

    // Acquire an instance of `Clang`
    let clang = Clang::new().unwrap();

    // Create a new `Index`
    let index = Index::new(&clang, false, true);

    // Parse a source file into a translation unit
    let tu = index.parser(filename).arguments(&args).parse().unwrap();

    match opts.subcmd {
        SubCommand::Create(crt) => create_subcommand(crt, tu),
        SubCommand::Update(upd) => update_subcommand(upd, tu),
        SubCommand::Dump(dmp) => dump_subcommand(dmp, tu),
    }
}

fn create_subcommand(crt: CreateOpts, tu: TranslationUnit) -> Result<(), String> {
    if let Some(class) = find_class_entity(&tu, crt.interface.as_ref().unwrap()) {
        let mock_class_name = &crt
            .mock
            .unwrap_or(format!("{}Mock", class.get_display_name().unwrap()));
        let mock_def = generate_mock_definition(&class, &mock_class_name);

        println!("{}", mock_def);
        Ok(())
    } else {
        Err(format!(
            "no interface class named `{}` was found in the specified translation unit",
            &crt.interface.unwrap()
        ))
    }
}

fn update_subcommand(_upd: UpdateOpts, _tu: TranslationUnit) -> Result<(), String> {
    Err("not yet implemented".to_string())
}

fn dump_subcommand(dmp: DumpOpts, tu: TranslationUnit) -> Result<(), String> {
    let entity = dmp
        .class
        .and_then(|class_name| find_class_entity(&tu, &class_name))
        .unwrap_or(tu.get_entity());
    print_ast(&entity);
    Ok(())
}
