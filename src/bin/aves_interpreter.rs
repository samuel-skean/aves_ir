use std::{
    fs::File,
    io::{self, stdin, BufReader, BufWriter, Read},
    os::fd::AsRawFd as _, process::{self, Stdio},
};

use aves_ir::{assemble, bindings, write_bytecode::write_bytecode};
use clap::Parser;

// TODO: This should have two mutually exclusive options: interpret and print.
// These should be mutually exclusive since they both print to standard out.
// Interpret prints the result of interpreting the program to standard out, and
// print prints the human-readable form of the program to standard out. I should
// also be able to do neither, and just produce the assembled output. deriving
// Command on an enum, and making that part of the struct, should do the trick.
#[derive(Parser)]
struct CliOptions {
    #[arg(short, long = "bytecode", required_unless_present("text_path"))]
    bytecode_path: Option<std::path::PathBuf>,
    #[arg(short, long = "text", required_unless_present("bytecode_path"))]
    // TODO: Better name.
    text_path: Option<std::path::PathBuf>,
    #[arg(short, long = "output-bytecode", requires("text_path"))]
    output_bytecode_path: Option<std::path::PathBuf>,
    #[arg(short, long)]
    print: bool,
}

fn main() -> io::Result<()> {
    let options = CliOptions::parse();

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
            unreachable!("Clap didn't do its job.")
        }
        CliOptions {
            bytecode_path: None,
            text_path: Some(text_path),
            output_bytecode_path,
            print,
        } => {
            // STRETCH: Make this streaming.
            let mut text_program = String::new();
            let text_program = if text_path == <&str as Into<std::path::PathBuf>>::into("-") {
                stdin().read_to_string(&mut text_program)?;
                text_program
            } else {
                let mut text_file = BufReader::new(File::open(text_path)?);
                text_file.read_to_string(&mut text_program)?;
                text_program
            };
            
            // It is not ideal that we're sometimes writing the bytecode twice when we could be doing so once.
            let prog = assemble::program(&text_program).expect("Parsing error.");
            if let Some(output_bytecode_path) = output_bytecode_path {
                let mut output_bytecode_file = BufWriter::new(File::create(output_bytecode_path)?);
                write_bytecode(&prog, &mut output_bytecode_file)?;
            }

            let mut child_cmd = process::Command::new(std::env::current_exe().expect("Can't find current executable."));
            if print {
                child_cmd.arg("--print");
            }
            child_cmd.args(["--bytecode", "-"]);
            let mut child = child_cmd.stdin(Stdio::piped()).spawn()?;
            let mut child_stdin = child.stdin.as_ref().expect("Could not get child's stdin.");
            write_bytecode(&prog,&mut child_stdin)
                    .expect("Could not write bytecode into child's stdin.");
            child.wait().expect("Child process (interpreter) failed.");
        }
        CliOptions {
            bytecode_path: Some(bytecode_path),
            text_path: None,
            print,
            ..
        } => {
            let bytecode_file;
            // Why is it okay to turn a `File` into a raw fd with just an immutable
            // reference to the file? You can definitely conceptually modify the
            // file through the raw fd...
            let bytecode_fd = if bytecode_path == <&str as Into<std::path::PathBuf>>::into("-") {
                0
            } else {
                bytecode_file = File::open(bytecode_path)?;
                bytecode_file.as_raw_fd()
            };
            unsafe {
                let c_ir_node = bindings::ir_list_read(bytecode_fd);
                if print {
                    bindings::ir_list_print(c_ir_node);
                } else {
                    bindings::interpret(c_ir_node);
                }
                bindings::free_list_ir(c_ir_node);
            }
        }
    };
    Ok(())
}
