use crate::ast_iterators::print_ast;
use crate::cli::*;
use crate::error::CLIError::*;
use crate::error::*;
use crate::fs_iterators::*;
use crate::mock_generation::*;
use clang::*;
use std::path::{Path, PathBuf};

pub struct MockeryApp<'i> {
    tu: TranslationUnit<'i>,
}

impl<'i> MockeryApp<'i> {
    pub fn new(index: &'i Index, opts: &MockeryOpts) -> CLIResult<Self> {
        let source_file = std::fs::canonicalize(match &opts.subcmd {
            SubCommand::Create(crt) => &crt.interface_source[..],
            SubCommand::Update(upd) => &upd.mock_source[..],
            SubCommand::Dump(dmp) => &dmp.source[..],
        })
        .map_err(|e| SourceFileNotFound(e))?;

        let compile_db_dir = opts
            .compile_commands
            .as_ref()
            .map(|s| Ok(PathBuf::from(s)))
            .unwrap_or_else(|| {
                find_compilation_database(source_file.parent().unwrap(), opts.search_radius)
            })
            .and_then(|dir| {
                std::fs::canonicalize(dir).map_err(|e| SpecifiedCompilationDatabaseNotFound(e))
            })?;

        compile_db_dir
            .join("compile_commands.json")
            .canonicalize()
            .map_err(|e| SpecifiedCompilationDatabaseNotFound(e))?;

        std::env::set_current_dir(&compile_db_dir).unwrap();

        let compile_db = CompilationDatabase::from_directory(&compile_db_dir).unwrap();

        let commands = compile_db
            .get_compile_commands(&source_file)
            .map_err(|()| CompileCommandNotFound(source_file.clone()))?;
        let command_vec = commands.get_commands();
        let command = command_vec
            .first()
            .ok_or(CompileCommandNotFound(source_file))?;
        let filename = command.get_filename();
        let args: Vec<_> = command
            .get_arguments()
            .into_iter()
            .filter(|a| PathBuf::from(a) != filename)
            .filter(|a| !["/Tc", "/TC", "/Tp", "/TP"].contains(&a.as_str()))
            .collect();

        // Parse a source file into a translation unit
        let tu = index
            .parser(filename)
            .arguments(&args)
            .parse()
            .map_err(|e| SourceError(e))?;

        Ok(MockeryApp { tu })
    }

    pub fn run_create(&self, crt: CreateOpts) -> CLIResult<()> {
        let interface_name = crt.interface.as_ref().map_or(
            Path::new(&crt.interface_source)
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap(),
            String::as_str,
        );
        if let Some(class) = find_class_entity(&self.tu, interface_name) {
            let mock_class_name = &crt
                .mock
                .unwrap_or(format!("{}Mock", class.get_display_name().unwrap()));
            let mock_def = generate_mock_definition(&class, &mock_class_name);

            println!("{}", mock_def);
            Ok(())
        } else {
            Err(InterfaceClassNotFound(interface_name.to_string()))
        }
    }

    pub fn run_update(&self, _upd: UpdateOpts) -> CLIResult<()> {
        Err(NotYetImplemented)
    }

    pub fn run_dump(&self, dmp: DumpOpts) -> CLIResult<()> {
        let entity = dmp
            .class
            .and_then(|class_name| find_class_entity(&self.tu, &class_name))
            .unwrap_or(self.tu.get_entity());
        print_ast(&entity);
        Ok(())
    }
}

fn find_compilation_database(starting_point: &Path, radius: usize) -> CLIResult<PathBuf> {
    FilesystemDirectoryNode {
        path: std::fs::canonicalize(starting_point).map_err(|e| {
            CompilationDatabaseSearchStartingPointNotFound(starting_point.to_path_buf(), e)
        })?,
    }
    .search(radius)
    .find(|path| path.join("compile_commands.json").exists())
    .ok_or(CompilationDatabaseSearchFailed)
}
