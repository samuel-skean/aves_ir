use std::{io::{BufWriter, Read}, process::Stdio, thread};

use ipc_channel::ipc::IpcError;
use serde::{Deserialize, Serialize};

use crate::{bindings, ir_definition::Instruction, write_bytecode::write_bytecode};

#[derive(Serialize, Deserialize, Debug)]
pub enum ProgramStackItem {
    Int(i64),
    String(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProgramStack(pub Vec<ProgramStackItem>);

// TODO: Conversion code from C stack to Rust stack.

pub fn interpret<'program, 'stack>(program: &'program [Instruction]) -> Result<(String, ProgramStack), IpcError> {
    let mut child_builder = mitosis::Builder::new();
    child_builder.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = child_builder.spawn((), |()| unsafe { interpret_c() });

    let mut program_output = String::new();

    let program_stack = {
        let child_stdin = child.stdin().take().expect("Failed to get child process stdin.");
        thread::scope(|scope| {
            scope.spawn(move || {
                write_bytecode(program, &mut BufWriter::new(child_stdin)).expect("Unable to write program to the process.");
            });
            child.stdout().as_mut().unwrap().read_to_string(&mut program_output).expect("Unable to read output from process.");
            child.join()
        })
    }?;


    Ok((program_output, program_stack))
}


unsafe fn interpret_c() -> ProgramStack {
    let c_ir_node = bindings::ir_list_read(0);
    bindings::interpret(c_ir_node);
    bindings::free_list_ir(c_ir_node);
    // TODO: Actually fetch the program stack from the C code.
    ProgramStack(Vec::new())
}