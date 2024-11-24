use crate::bindings::*;
use std::io;

use crate::ir_definition::{Intrinsic, Label, IrNode};


pub fn dump_bytecode(ir_list: &[IrNode], out: &mut impl io::Write) -> io::Result<()> {
    for node in ir_list {
        node.write_bytecode(out)?;
    }
    Ok(())
}

trait WriteBytecode {
    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()>;
}

impl WriteBytecode for i32 {
    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        out.write_all(&self.to_le_bytes())
    }
}

impl WriteBytecode for u32 {
    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        out.write_all(&self.to_le_bytes())
    }
}

impl WriteBytecode for i64 {
    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        // Should we really be limiting ourselves to only 32 bits for integer constants in the IR?
        // I guess if we're mostly targeting MIPS-32, that makes sense.
        (*self as u32).write_bytecode(out)
    }
}

// TODO: This is a little ugly because it's *so close* to a name collision with the C stuff.
impl WriteBytecode for Label {

    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        let raw_bytes = self.name().as_bytes();

        // TODO: But why is it signed? Is it safe to make it unsigned?
        let length_including_null_terminator = (raw_bytes.len() + 1) as i32;
        length_including_null_terminator.write_bytecode(out)?;
        out.write_all(raw_bytes)?;
        out.write_all(&[0u8])
    }
}

impl WriteBytecode for Intrinsic {

    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        let val_to_write = match self {
            Intrinsic::PrintInt => intrinsic_intrinsic_print_int,
            Intrinsic::PrintString => intrinsic_intrinsic_print_string,
            Intrinsic::Exit => intrinsic_intrinsic_exit,
        };
        val_to_write.write_bytecode(out)
    }
}

impl WriteBytecode for IrNode {
    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        match self {
            IrNode::Nop => ir_op_ir_nop.write_bytecode(out),
            IrNode::Iconst(num) => { 
                ir_op_ir_iconst.write_bytecode(out)?;
                num.write_bytecode(out)
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
                ir_op_ir_lbl.write_bytecode(out)?;
                label.write_bytecode(out)
            }
            IrNode::Jump(label) => {
                ir_op_ir_jump.write_bytecode(out)?;
                label.write_bytecode(out)
            }
            IrNode::BranchZero(label) => todo!(),
            IrNode::Function { label, num_locs } => todo!(),
            IrNode::Call { label, num_args } => todo!(),
            IrNode::Ret => todo!(),
            IrNode::Intrinsic(intrinsic) => {
                ir_op_ir_intrinsic.write_bytecode(out)?;
                intrinsic.write_bytecode(out)
            }
            IrNode::Push { reg } => todo!(),
            IrNode::Pop { reg } => todo!(),
        }
    }
}