use nom::{branch::alt, bytes::complete::{tag_no_case, take_while}, character::{complete::{char, space1}, is_alphanumeric}, combinator::{all_consuming, opt}, multi::separated_list0, sequence::tuple, IResult};

use crate::ir_definition::{IrNode, Label};
type NodeResult<'a> = IResult<&'a [u8], IrNode>;

fn identifier(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while(|c| is_alphanumeric(c) || c == b'$' || c == b'_')(input)
}

fn jump(input: &[u8]) -> NodeResult {
    let (rest, _) = tuple((tag_no_case(b"JUMP"), space1))(input)?;
    let (rest, label_text) = identifier(rest)?;

    Ok((rest, IrNode::Jump(Label(label_text.into()))))
}

// No-arg nodes:
// TODO: These should be done through a macro, but I don't know how to macro right now.
// Could also be a function that returns a function, but when I tried to write that it had to copy the IrNode.
fn nop(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"NOP")(input)?;
    Ok((rest, IrNode::Nop))
}

fn add(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"ADD")(input)?;
    Ok((rest, IrNode::Add))
}

fn sub(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"SUB")(input)?;
    Ok((rest, IrNode::Sub))
}

fn mul(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"MUL")(input)?;
    Ok((rest, IrNode::Mul))
}

fn div(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"DIV")(input)?;
    Ok((rest, IrNode::Div))
}

fn mod_(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"MOD")(input)?;
    Ok((rest, IrNode::Mod))
}

fn bor(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"BOR")(input)?;
    Ok((rest, IrNode::Bor))
}

fn band(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"BAND")(input)?;
    Ok((rest, IrNode::Band))
}

fn xor(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"XOR")(input)?;
    Ok((rest, IrNode::Xor))
}

fn or(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"OR")(input)?;
    Ok((rest, IrNode::Or))
}

fn and(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"AND")(input)?;
    Ok((rest, IrNode::And))
}

fn eq(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"EQ")(input)?;
    Ok((rest, IrNode::Eq))
}

fn lt(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"LT")(input)?;
    Ok((rest, IrNode::Lt))
}

fn gt(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"GT")(input)?;
    Ok((rest, IrNode::Gt))
}

fn not(input: &[u8]) -> NodeResult {
    let (rest, _) = tag_no_case(b"NOT")(input)?;
    Ok((rest, IrNode::Not))
}

pub fn node(input: &[u8]) -> NodeResult {
    alt((jump, nop, add, sub, mul, div, mod_, bor, band, xor, or, and, eq, lt, gt, not))(input)
}

pub fn program(input: &[u8]) -> Result<Vec<IrNode>, nom::Err<nom::error::Error<&[u8]>>> {
    // TODO: Handle the final missing newline. This somehow doesn't work.
    let (rest, (prog, _final_newline)) = 
        all_consuming(tuple((separated_list0(char('\n'), node), opt(char('\n')))))(input)?;
    assert_eq!(rest, &b""[..]);
    Ok(prog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jump() {
        assert_eq!(node(b"JUMP L0  h"), Ok((&b"  h"[..], IrNode::Jump(Label(b"L0".into())))));
        assert_eq!(node(b"JUMP alskdhjfa"), Ok((&b""[..], IrNode::Jump(Label(b"alskdhjfa".into())))));
    }

    #[test]
    fn noarg_nodes() {
        // I never know how many tests to write...
        // Positive examples:
        assert_eq!(node(b"ADD "), Ok((&b" "[..], IrNode::Add)));
        assert_eq!(node(b"NOP"), Ok((&b""[..], IrNode::Nop)));
        assert_eq!(node(b"sUb   kdf"), Ok((&b"   kdf"[..], IrNode::Sub)));
        assert_eq!(node(b"Mul "), Ok((&b" "[..], IrNode::Mul)));
        assert_eq!(node(b"diV  "), Ok((&b"  "[..], IrNode::Div)));
        assert_eq!(node(b"mod  $$04"), Ok((&b"  $$04"[..], IrNode::Mod)));
        assert_eq!(node(b"BOR      \n"), Ok((&b"      \n"[..], IrNode::Bor)));
        assert_eq!(node(b"bANd  "), Ok((&b"  "[..], IrNode::Band)));
        assert_eq!(node(b"xor"), Ok((&b""[..], IrNode::Xor)));
        assert_eq!(node(b"or"), Ok((&b""[..], IrNode::Or)));
        assert_eq!(node(b"and"), Ok((&b""[..], IrNode::And)));
        assert_eq!(node(b"eq"), Ok((&b""[..], IrNode::Eq)));
        assert_eq!(node(b"lT"), Ok((&b""[..], IrNode::Lt)));
        assert_eq!(node(b"gt"), Ok((&b""[..], IrNode::Gt)));
        assert_eq!(node(b"Not"), Ok((&b""[..], IrNode::Not)));

        // Negative examples:
        assert!(node(b"n ot").is_err());
        assert!(node(b" div").is_err()); // Doesn't accept leading whitespace.
        assert_ne!(node(b"bor   "), Ok((&b""[..], IrNode::Bor))); // Doesn't accept trailing whitespace.
    }

    #[test]
    fn simple_program() {
        assert_eq!(
            program(b"band\n\
                     bor\n\
                     and\n\
                     xor"), // Works without terminating newline.
            Ok(vec![
                IrNode::Band,
                IrNode::Bor,
                IrNode::And,
                IrNode::Xor,
            ])
        );
        assert_eq!(
            program(b"band\n\
                     BOR\n\
                     And\n\
                     xOR\n"), // Also works with terminating newline.
            Ok(vec![
                IrNode::Band,
                IrNode::Bor,
                IrNode::And,
                IrNode::Xor,
            ])
        );
    }
}

// TODO: Each instruction function should not take trailing whitespace. That should be left to the thing that processes multiple instructions, that can take newlines and spaces. I think instructions could totally legally be on the same line!