use std::{
	env::{self, Args},
	fs,
	path::PathBuf,
	process::ExitCode,
};

use bean_script::{
	error::{BeanResult, ErrorSource},
	evaluator, lexer,
	modules::{
		registry::{ModuleRegistry, RegistryFeatures},
		CustomModule,
	},
	parser,
	util::make_ref,
};

mod interactive_terminal;

const HELP_MSG: &str = "Function-based language interpreter.
Usage: beans [OPTIONS] [PATH]

Options:
	-p, --parse     Parse file without evaluating it.
	-l, --tokenize  Tokenize file without parsing it.
	-h, --help      Print this message and exit.
	-i, --stdin     Interpret input from stdin.";

struct CliArgs {
	no_args: bool,
	f_help: bool,
	f_parse: bool,
	f_tokenize: bool,
	f_stdin: bool,
	path: Option<String>,
}

fn main() -> ExitCode {
	let args = parse_args(env::args());
	if args.no_args || args.f_help {
		println!("{}", HELP_MSG);
		ExitCode::SUCCESS
	} else if args.f_stdin {
		let result = interactive_terminal::open();
		if let rustyline::Result::Err(error) = result {
			println!(
				"\x1b[31;1merror\x1b[0m: Failed to parse stdin.\n\t{:?}",
				error
			);
			ExitCode::FAILURE
		} else {
			ExitCode::SUCCESS
		}
	} else {
		let path_str = args.path.expect("Expected path to file.");
		let file = fs::read_to_string(path_str.clone()).expect("Failed to open file");

		let tokens = lexer::tokenize(file);
		if args.f_tokenize {
			dbg!(tokens);
			return ExitCode::SUCCESS;
		}

		let tree = parser::parse(tokens);
		if let Err(error) = tree {
			println!(
				"\x1b[31;1merror\x1b[0m: {}",
				error.trace(ErrorSource::File(path_str.clone()))
			);
			return ExitCode::FAILURE;
		}
		let tree = tree.unwrap();

		if args.f_parse {
			dbg!(tree);
			return ExitCode::SUCCESS;
		}

		let mut dir_path = PathBuf::from(path_str.clone());
		dir_path.pop();

		let registry = make_ref(ModuleRegistry::new(RegistryFeatures::default()));
		let program_scope = CustomModule::new(registry, dir_path);
		let result = evaluator::evaluate(&tree, make_ref(program_scope));
		if let Err(error) = result {
			println!(
				"\x1b[31;1merror\x1b[0m: {}",
				error.trace(ErrorSource::File(path_str.clone()))
			);
			ExitCode::FAILURE
		} else {
			ExitCode::SUCCESS
		}
	}
}

fn parse_args(mut args: Args) -> CliArgs {
	let no_args = args.len() == 1;
	let mut flags: Vec<String> = Vec::new();
	let mut path: Option<String> = None;

	args.next();
	for arg in args {
		if arg.starts_with("--") || arg.starts_with("-") {
			flags.push(arg);
		} else if path.is_none() {
			path = Some(arg);
		}
	}

	CliArgs {
		no_args,
		path,
		f_help: flags.contains(&String::from("--help"))
			|| flags.contains(&String::from("-h")),
		f_parse: flags.contains(&String::from("--parse"))
			|| flags.contains(&String::from("-p")),
		f_tokenize: flags.contains(&String::from("--tokenize"))
			|| flags.contains(&String::from("-l")),
		f_stdin: flags.contains(&String::from("--stdin"))
			|| flags.contains(&String::from("-i")),
	}
}
