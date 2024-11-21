// TODO: Make all String's &str. Requires lifetime shenanigans.
pub struct Label {
    name: String,
}

pub enum Intrinsic {
    PrintInt,
    PrintString,
    Exit,
}

pub enum IrNode {
    Nop,

    // Arithmetic/logic operations:
    Iconst(u64),
    Sconst(String),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Bor,
    Band,
    Xor,
    Or,
    And,
    Eq,
    Lt,
    Gt,
    Not,

    // Variables
    ReserveString {size: u64, name: String, initial_value: String},
    ReserveInt(String),
    Read(String),
    Write(String),
    ArgLocalRead(u64),
    ArgLocalWrite(u64),

    // Control-flow
    Label(Label),
    Jump(Label),
    BranchZero(Label),

    // Functions
    Function {label: Label, num_locs: u64},
    Call {label: Label, num_vars: u64},
    Ret,
    Intrinsic(Intrinsic),

    // TODO: These have registers specified as immediates. What's up with that?
    Push {reg: u64}, // I don't think Bluejay would ever generate these.
    Pop {reg: u64},

}