use std::{
    fs::File,
    io::{self, BufReader, BufWriter, Read},
    os::fd::AsRawFd as _,
};

use aves_ir::{assemble, bindings, dump_bytecode::dump_bytecode};
use clap::Parser;

#[derive(Parser)]
struct CliOptions {
    #[arg(short, long = "bytecode", required_unless_present("text_path"))]
    bytecode_path: Option<std::path::PathBuf>,
    #[arg(short, long = "text", required_unless_present("bytecode_path"))]
    // TODO: Better name.
    text_path: Option<std::path::PathBuf>,
    #[arg(short, long = "output-bytecode")]
    output_bytecode_path: Option<std::path::PathBuf>,
    #[arg(short, long)]
    print: bool,
}

fn main() -> io::Result<()> {
    let options = CliOptions::parse();

    let mut prog = Some(Vec::new());
    match options {
        CliOptions {
            bytecode_path: Some(_),
            text_path: Some(_),
            ..
        } => {
            panic!("Can't specify both formats to read from!");
        }
        CliOptions {
            bytecode_path: None,
            text_path: None,
            ..
        } => {
            unreachable!()
        }
        CliOptions {
            bytecode_path: None,
            text_path: Some(text_path),
            ..
        } => {
            // STRETCH: Make this streaming.
            let mut text_file = BufReader::new(File::open(text_path)?);
            let mut text_program = String::new();
            text_file.read_to_string(&mut text_program)?;
            prog = Some(assemble::program(&text_program).expect("Parsing error."));
            // TODO: Interpret the code.
            println!("Program was: {:?}", prog);
        }
        CliOptions {
            bytecode_path: Some(bytecode_path),
            text_path: None,
            print,
            ..
        } => {
            let bytecode_file = File::open(bytecode_path)?;
            // Why is it okay to turn a `File` into a raw fd with just an immutable
            // reference to the file? You can definitely conceptually modify the
            // file through the raw fd...
            let bytecode_fd = bytecode_file.as_raw_fd();

            unsafe {
                let c_ir_node = bindings::ir_list_read(bytecode_fd);
                if print {
                    bindings::ir_list_print(c_ir_node);
                } else {
                    todo!("Interpreting from the Rust program is not implemented yet!");
                }
            }
        }
    }

    if let Some(output_bytecode_path) = options.output_bytecode_path {
        let mut output_bytecode_file = BufWriter::new(File::create(output_bytecode_path)?);
        dump_bytecode(&prog.unwrap(), &mut output_bytecode_file)?;
    }

    Ok(())
}
