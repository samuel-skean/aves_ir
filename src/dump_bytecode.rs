use crate::bindings::*;
use std::io;

use crate::ir_definition::{Intrinsic, Label, IrNode};

trait ToBytecode {
    type Output: AsRef<[u8]>;
    fn to_bytecode(&self) -> Self::Output;
}

impl ToBytecode for i32 {
    // TODO: Make this not require an allocation.
    type Output = Vec<u8>;

    fn to_bytecode(&self) -> Self::Output {
        self.to_le_bytes().into()
    }
}

impl ToBytecode for u32 {
    // TODO: make this not require an allocation.

    type Output = Vec<u8>;

    fn to_bytecode(&self) -> Self::Output {
        self.to_le_bytes().into()
    }
}

impl ToBytecode for i64 {
    // TODO: make this not require an allocation.
    type Output = Vec<u8>;

    fn to_bytecode(&self) -> Self::Output {
        // Should we really be limiting ourselves to only 32 bits for integer constants in the IR?
        // I guess if we're mostly targeting MIPS-32, that makes sense.
        (*self as i32).to_le_bytes().into()
    }
}

// TODO: This is a little ugly because it's *so close* to a name collision with the C stuff.
impl ToBytecode for Label {
    type Output = Vec<u8>;
    fn to_bytecode(&self) -> Self::Output {
        let raw_bytes = self.name().as_bytes();

        // TODO: But why is it signed? Is it safe to make it unsigned?
        let length_including_null_terminator = (raw_bytes.len() + 1) as i32;
        let mut bytecode = Vec::from(length_including_null_terminator.to_bytecode());
        bytecode.extend_from_slice(raw_bytes);
        bytecode.push(0u8);
        bytecode
    }
}

impl ToBytecode for Intrinsic {
    // TODO: Make this not require an allocation.
    type Output = Vec<u8>;
    fn to_bytecode(&self) -> Self::Output {
        // STRETCH: *sigh*. Needed to do this because I couldn't borrow the
        // result of to_le_bytes and return it from this function.
        // STRETCH: Somehow make this use to_bytecode on the ints as well.
        // Should be easier if I could make to_bytecode not return a Vec for the
        // ints.
        const INTRINSIC_BYTES: [&[u8; 4]; 3] = {
            let mut intrinsic_bytes = [&[0u8; 4]; 3];
            intrinsic_bytes[intrinsic_intrinsic_print_int as usize] = &intrinsic_intrinsic_print_int.to_le_bytes();
            intrinsic_bytes[intrinsic_intrinsic_print_string as usize] = &intrinsic_intrinsic_print_string.to_le_bytes();
            intrinsic_bytes[intrinsic_intrinsic_exit as usize] = &intrinsic_intrinsic_exit.to_le_bytes();
            intrinsic_bytes
        };
        INTRINSIC_BYTES[match self {
            Intrinsic::PrintInt => intrinsic_intrinsic_print_int,
            Intrinsic::PrintString => intrinsic_intrinsic_print_string,
            Intrinsic::Exit => intrinsic_intrinsic_exit,
        } as usize].into()
    }
}
// TODO: Consider making the actual conversion into bytecode by implementing ToBytecode on IrNode.

pub fn dump_bytecode(ir_list: &[IrNode], mut out: impl io::Write) -> io::Result<()> {
    for node in ir_list {
        match node {
            IrNode::Nop => { out.write_all(&ir_op_ir_nop.to_bytecode())?; }
            IrNode::Iconst(num) => { 
                out.write_all(&ir_op_ir_iconst.to_bytecode())?;
                out.write_all(&num.to_bytecode())?;
            }
            IrNode::Sconst(_) => todo!(),
            IrNode::Add => todo!(),
            IrNode::Sub => todo!(),
            IrNode::Mul => todo!(),
            IrNode::Div => todo!(),
            IrNode::Mod => todo!(),
            IrNode::Bor => todo!(),
            IrNode::Band => todo!(),
            IrNode::Xor => todo!(),
            IrNode::Or => todo!(),
            IrNode::And => todo!(),
            IrNode::Eq => todo!(),
            IrNode::Lt => todo!(),
            IrNode::Gt => todo!(),
            IrNode::Not => todo!(),
            IrNode::ReserveString { size, name, initial_value } => todo!(),
            IrNode::ReserveInt { name } => todo!(),
            IrNode::Read(_) => todo!(),
            IrNode::Write(_) => todo!(),
            IrNode::ArgLocalRead(_) => todo!(),
            IrNode::ArgLocalWrite(_) => todo!(),
            IrNode::Label(label) => {
                out.write_all(&ir_op_ir_lbl.to_bytecode())?;
                out.write_all(&label.to_bytecode())?;
            }
            IrNode::Jump(label) => {
                out.write_all(&ir_op_ir_jump.to_bytecode())?;
                out.write_all(&label.to_bytecode())?;
            }
            IrNode::BranchZero(label) => todo!(),
            IrNode::Function { label, num_locs } => todo!(),
            IrNode::Call { label, num_args } => todo!(),
            IrNode::Ret => todo!(),
            IrNode::Intrinsic(intrinsic) => {
                out.write_all(&ir_op_ir_intrinsic.to_bytecode())?;
                out.write_all(&intrinsic.to_bytecode())?;
            }
            IrNode::Push { reg } => todo!(),
            IrNode::Pop { reg } => todo!(),
        };
    }
    Ok(())
}