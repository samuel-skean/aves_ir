use crate::bindings::*;
use std::io;

use crate::ir_definition::{Intrinsic, IrNode, Label};

pub fn write_bytecode(ir_list: &[IrNode], out: &mut impl io::Write) -> io::Result<()> {
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
        i32::try_from(*self)
            .expect("Integer too big for serialized bytecode format.")
            .write_bytecode(out)
    }
}

impl WriteBytecode for u64 {
    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        // This is an i32 on purpose, because the C code expects an int, not an unsigned int.
        i32::try_from(*self)
            .expect("Integer too big for serialized bytecode format.")
            .write_bytecode(out)
    }
}

impl WriteBytecode for &str {
    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        let raw_bytes = self.as_bytes();

        // TODO: But why is it signed? Is it safe to make it unsigned?
        let length_including_null_terminator = i32::try_from(raw_bytes.len() + 1)
            .expect("String too long for serialized bytecode format.");
        length_including_null_terminator.write_bytecode(out)?;
        out.write_all(raw_bytes)?;
        out.write_all(&[0u8])
    }
}

// TODO: `use`ing Label and Intrinsic is a little ugly because it's *so close*
// to a name collision with the C stuff.
impl WriteBytecode for Label {
    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        self.name().write_bytecode(out)
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
// TODO: consider creating newtyping bindings for enums in ir.c instead, and then
// importing all the variants, to cut down on noise.
impl WriteBytecode for IrNode {
    fn write_bytecode(&self, out: &mut impl io::Write) -> io::Result<()> {
        match self {
            IrNode::Nop => ir_op_ir_nop.write_bytecode(out),
            IrNode::Iconst(num) => {
                ir_op_ir_iconst.write_bytecode(out)?;
                num.write_bytecode(out)
            }
            IrNode::Sconst(text) => {
                ir_op_ir_sconst.write_bytecode(out)?;
                text.as_str().write_bytecode(out)
            }
            IrNode::Add => ir_op_ir_add.write_bytecode(out),
            IrNode::Sub => ir_op_ir_sub.write_bytecode(out),
            IrNode::Mul => ir_op_ir_mul.write_bytecode(out),
            IrNode::Div => ir_op_ir_div.write_bytecode(out),
            IrNode::Mod => ir_op_ir_mod.write_bytecode(out),
            IrNode::Bor => ir_op_ir_bor.write_bytecode(out),
            IrNode::Band => ir_op_ir_band.write_bytecode(out),
            IrNode::Xor => ir_op_ir_xor.write_bytecode(out),
            IrNode::Or => ir_op_ir_or.write_bytecode(out),
            IrNode::And => ir_op_ir_and.write_bytecode(out),
            IrNode::Eq => ir_op_ir_eq.write_bytecode(out),
            IrNode::Lt => ir_op_ir_lt.write_bytecode(out),
            IrNode::Gt => ir_op_ir_gt.write_bytecode(out),
            IrNode::Not => ir_op_ir_not.write_bytecode(out),
            IrNode::ReserveString {
                size,
                name,
                initial_value,
            } => {
                ir_op_ir_reserve.write_bytecode(out)?;
                name.as_str().write_bytecode(out)?;
                initial_value.as_str().write_bytecode(out)?;
                size.write_bytecode(out)
            }
            IrNode::ReserveInt { name } => {
                ir_op_ir_reserve.write_bytecode(out)?;
                name.as_str().write_bytecode(out)?;
                // Write the size 0, and nothing else for the string, because the string is conceptually null.
                0.write_bytecode(out)?;
                4.write_bytecode(out)
            }
            IrNode::Read(name) => {
                ir_op_ir_read.write_bytecode(out)?;
                name.as_str().write_bytecode(out)
            }
            IrNode::Write(name) => {
                ir_op_ir_write.write_bytecode(out)?;
                name.as_str().write_bytecode(out)
            }
            IrNode::ArgLocalRead(index) => {
                ir_op_ir_arglocal_read.write_bytecode(out)?;
                index.write_bytecode(out)
            }
            IrNode::ArgLocalWrite(index) => {
                ir_op_ir_arglocal_write.write_bytecode(out)?;
                index.write_bytecode(out)
            }
            IrNode::Label(label) => {
                ir_op_ir_lbl.write_bytecode(out)?;
                label.write_bytecode(out)
            }
            IrNode::Jump(label) => {
                ir_op_ir_jump.write_bytecode(out)?;
                label.write_bytecode(out)
            }
            IrNode::BranchZero(label) => {
                ir_op_ir_branchzero.write_bytecode(out)?;
                label.write_bytecode(out)
            }
            IrNode::Function { label, num_locs } => {
                ir_op_ir_function.write_bytecode(out)?;
                label.write_bytecode(out)?;
                num_locs.write_bytecode(out)
            }
            IrNode::Call { label, num_args } => {
                ir_op_ir_call.write_bytecode(out)?;
                label.write_bytecode(out)?;
                num_args.write_bytecode(out)
            }
            IrNode::Ret => ir_op_ir_ret.write_bytecode(out),
            IrNode::Intrinsic(intrinsic) => {
                ir_op_ir_intrinsic.write_bytecode(out)?;
                intrinsic.write_bytecode(out)
            }
            IrNode::Push { reg } => {
                ir_op_ir_push.write_bytecode(out)?;
                reg.write_bytecode(out)
            }
            IrNode::Pop { reg } => {
                ir_op_ir_pop.write_bytecode(out)?;
                reg.write_bytecode(out)
            }
        }
    }
}
