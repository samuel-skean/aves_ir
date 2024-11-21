// TODO: Make all Vec<u8>'s &str. Requires lifetime shenanigans.
#[derive(Debug, PartialEq)]
pub struct Label(pub Vec<u8>);

#[derive(Debug, PartialEq,)]
pub enum Intrinsic {
    PrintInt,
    PrintString,
    Exit,
}

#[derive(Debug, PartialEq)]
pub enum IrNode {
    Nop,

    // Arithmetic/logic operations:
    Iconst(u64),
    Sconst(Vec<u8>),
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
    ReserveString {size: u64, name: Vec<u8>, initial_value: Vec<u8>},
    ReserveInt(Vec<u8>),
    Read(Vec<u8>),
    Write(Vec<u8>),
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