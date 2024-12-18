// TODO: Make all String's &str. Requires lifetime shenanigans.
#[derive(Debug, PartialEq)]
pub struct Label(String);

impl Label {
    pub fn named(name: &str) -> Self {
        Label(String::from(name))
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Intrinsic {
    PrintInt,
    PrintString,
    Exit,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Nop,

    // Arithmetic/logic operations:
    Iconst(i64),
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
    ReserveString {
        size: u64,
        name: String,
        initial_value: String,
    },
    ReserveInt {
        name: String,
    },
    Read(String),
    Write(String),
    ArgLocalRead(u64),
    ArgLocalWrite(u64),

    // Control-flow
    Label(Label), // I guess labels are a kind of instruction - a no-op that also indicates where things are.
    Jump(Label),
    BranchZero(Label),

    // Functions
    Function {
        label: Label,
        num_locs: u64,
    },
    Call {
        label: Label,
        num_args: u64,
    },
    Ret,
    Intrinsic(Intrinsic),

    // TODO: These have registers specified as immediates. What's up with that?
    Push {
        reg: i64,
    }, // I don't think Bluejay would ever generate these.
    Pop {
        reg: i64,
    },
}
