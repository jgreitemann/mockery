mod ast_iterators;
mod cli;
mod fs_iterators;
mod mock_generation;
mod test_utils;

use crate::ast_iterators::print_ast;
use crate::cli::*;
use crate::fs_iterators::*;
use crate::mock_generation::*;
use clang::*;
use clap::Clap;
use std::path::{Path, PathBuf};

fn find_compilation_database(
    starting_point: &Path,
    radius: usize,
) -> Result<CompilationDatabase, ()> {
    FilesystemDirectoryNode {
        path: std::fs::canonicalize(starting_point).map_err(|_| ())?,
    }
    .search(radius)
    .find(|path| path.join("compile_commands.json").exists())
    .and_then(|path| CompilationDatabase::from_directory(&path).ok())
    .ok_or(())
}

fn main() {
    let opts = MockeryOpts::parse();

    let source_file = match &opts.subcmd {
        SubCommand::Create(crt) => &crt.interface_source[..],
        SubCommand::Update(upd) => &upd.mock_source[..],
        SubCommand::Dump(dmp) => &dmp.source[..],
    };

    let compile_db = match opts.compile_commands {
        Some(path) => CompilationDatabase::from_directory(path),
        None => {
            find_compilation_database(Path::new(source_file).parent().unwrap(), opts.search_radius)
        }
    }
    .unwrap();

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
        SubCommand::Create(crt) => {
            if let Some(class) = find_class_entity(&tu, &crt.interface.unwrap()) {
                let mock_class_name = crt
                    .mock
                    .unwrap_or(format!("{}Mock", class.get_display_name().unwrap()));
                let mock_def = generate_mock_definition(&class, &mock_class_name);

                println!("{}", mock_def);
            }
        }
        SubCommand::Update(_) => {
            panic!()
        }
        SubCommand::Dump(dmp) => {
            let entity = dmp
                .class
                .and_then(|class_name| find_class_entity(&tu, &class_name))
                .unwrap_or(tu.get_entity());
            print_ast(&entity);
        }
    }
}
