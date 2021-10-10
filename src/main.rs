mod ast_iterators;
mod mock_generation;
mod test_utils;

use crate::mock_generation::*;
use crate::ast_iterators::print_ast;
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
    println!("args: {:?}", args);

    // Acquire an instance of `Clang`
    let clang = Clang::new().unwrap();

    // Create a new `Index`
    let index = Index::new(&clang, false, true);

    // Parse a source file into a translation unit
    let tu = index.parser(filename).arguments(&args).parse().unwrap();

    print_ast(&tu.get_entity());

    if let Some(class) = find_class_entity(&tu, "Fooable") {
        let mock_class_name = format!("{}Mock", class.get_display_name().unwrap());
        let mock_def = generate_mock_definition(&class, &mock_class_name);

        println!("{}", mock_def);
    }
}
