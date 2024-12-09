use std::{collections::VecDeque, io::{BufWriter, Read}, process::Stdio, ptr::null_mut, thread};

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

impl TryFrom<*mut bindings::stack_node> for ProgramStack {
    type Error = std::str::Utf8Error;

    #[allow(non_upper_case_globals)]
    fn try_from(c_stack_head: *mut bindings::stack_node) -> Result<Self, Self::Error> {
        use bindings::{stack_data_stack_int, stack_data_stack_str};
        use std::ffi::CStr;
        
        let mut curr = c_stack_head;
        // TODO: This stack is in the opposite order of what it would be if it
        // were created in Rust, since we're copying from the top. Use a
        // VecDeque, push on the front, and then collect it into a Vec. That
        // oughta be about as fast as building something literally from the
        // back, and maybe there are options on VecDeque that can make it
        // literally that fast.
        let mut stack_items = VecDeque::new();
        while curr != null_mut() {
            // # SAFETY: c_stack_node is not null.
            match unsafe { (*curr).kind } {
                stack_data_stack_int => {
                    // # SAFETY: Trusting the C code to set the kind correctly...
                    let i = unsafe { (*curr).data.stack_int };
                    stack_items.push_front(ProgramStackItem::Int(i as _));
                }
                stack_data_stack_str => {
                    // # SAFETY: Trusting the C code to set the kind correctly...
                    let c_str = unsafe { CStr::from_ptr((*curr).data.stack_str) };
                    let s = c_str.to_str()?.to_owned();
                    stack_items.push_front(ProgramStackItem::String(s));
                }
                kind => {
                    panic!("Invalid stack_item kind: {kind}! Bad C code!");
                }
            }
            // # SAFETY: c_stack_node is not null.
            unsafe { curr = (*curr).next; }
        };
        unsafe { bindings::free_stack(c_stack_head); }
        Ok(ProgramStack(Vec::from(stack_items)))

    }
}

pub fn interpret<'program>(program: &'program [Instruction]) -> Result<(String, ProgramStack), IpcError> {
    let mut child_builder = mitosis::Builder::new();
    child_builder.stdin(Stdio::piped()).stdout(Stdio::piped());
    let mut child = child_builder.spawn((), |()| unsafe { interpret_in_c() });

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


unsafe fn interpret_in_c() -> ProgramStack {
    let c_ir_node = bindings::ir_list_read(0);
    // TODO: Somehow encode that the head of this owns the list.
    let mut c_program_stack: *mut bindings::stack_node = null_mut();
    bindings::interpret(c_ir_node, &mut c_program_stack as _);
    dbg!(c_program_stack);
    let program_stack = ProgramStack::try_from(c_program_stack).expect("The strings in the stack weren't all UTF-8!");
    // This must be done *after* the ProgramStack is constructed, otherwise the strings will be freed while still referenced.
    bindings::free_list_ir(c_ir_node);
    program_stack
}