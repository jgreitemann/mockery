extern crate clang;

use clang::*;
use std::path::PathBuf;

fn main() {
    let compile_db = CompilationDatabase::from_directory("examples/cmake-build-debug").unwrap();
    let commands = compile_db
        .get_compile_commands("examples/prog.cpp")
        .unwrap();
    let command_vec = commands.get_commands();
    let command = command_vec.first().unwrap();
    let filename = command.get_filename();
    let args: Vec<_> = command
        .get_arguments()
        .into_iter()
        .filter(|a| PathBuf::from(a) != filename)
        .collect();

    println!("filename: {:?}", filename);

    // Acquire an instance of `Clang`
    let clang = Clang::new().unwrap();

    // Create a new `Index`
    let index = Index::new(&clang, false, true);

    // Parse a source file into a translation unit
    let tu = index.parser(filename).arguments(&args).parse().unwrap();

    print_ast(&tu.get_entity(), 0);
}

fn print_ast(e: &Entity, indentation_level: usize) {
    println!(
        "{}{:?}: {:?}",
        "\t".repeat(indentation_level),
        e.get_name().unwrap_or("".to_string()),
        e.get_kind()
    );
    for e in e.get_children().into_iter() {
        print_ast(&e, indentation_level + 1);
    }
}
