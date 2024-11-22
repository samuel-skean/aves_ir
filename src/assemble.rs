use nom::{
    branch::alt,
    bytes::complete::{escaped, tag_no_case, take_while},
    character::{
        complete::{multispace0, multispace1, none_of, one_of, space1, u64 as nom_u64},
        is_alphanumeric,
    },
    combinator::all_consuming,
    multi::separated_list0,
    sequence::{delimited, terminated},
    IResult,
};

use crate::ir_definition::{IrNode, Label};
type NodeResult<'a> = IResult<&'a [u8], IrNode>;

fn identifier(input: &[u8]) -> IResult<&[u8], &[u8]> {
    // TODO: Make this require there to be at least one thing in the input.
    // TODO: Could I do this with permutation, and still use strs?
    take_while(|c| is_alphanumeric(c) || c == b'$' || c == b'_')(input)
}

fn iconst(input: &[u8]) -> NodeResult {
    let (rest, _) = terminated(tag_no_case("ICONST"), space1)(input)?;
    let (rest, num) = nom_u64(rest)?;
    Ok((rest, IrNode::Iconst(num)))
}

fn sconst(input: &[u8]) -> NodeResult {
    let (rest, _) = terminated(tag_no_case(b"SCONST"), space1)(input)?;
    // TODO: This doesn't actually remove the \n. I need to use
    // escaped_transform to do that, but it really seems to want to produce
    // strings - probably because the control_char must be a char. Luckily, I
    // should be able to get the Vec inside that String.
    let (rest, transformed_text) = escaped(none_of(&b"\\"[..]), '\\', one_of(&b"n"[..]))(rest)?;
    println!("{rest:?}");

    Ok((rest, IrNode::Sconst(transformed_text.into())))
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

fn jump(input: &[u8]) -> NodeResult {
    let (rest, _) = terminated(tag_no_case(b"JUMP"), space1)(input)?;
    let (rest, label_text) = identifier(rest)?;

    Ok((rest, IrNode::Jump(Label(label_text.into()))))
}

pub fn node(input: &[u8]) -> NodeResult {
    alt((
        iconst, sconst, nop, add, sub, mul, div, mod_, bor, band, xor, or, and, eq, lt, gt, not,
        jump,
    ))(input)
}

pub fn program(input: &[u8]) -> Result<Vec<IrNode>, nom::Err<nom::error::Error<&[u8]>>> {
    // TODO: Handle the final missing newline. This somehow doesn't work.
    let (rest, prog) = all_consuming(delimited(
        multispace0,
        separated_list0(multispace1, node),
        multispace0,
    ))(input)?;
    assert_eq!(rest, &b""[..]); // Surely this is redundant because of how all-consuming works.
    Ok(prog)
}

#[cfg(test)]
mod tests {
    // TODO: Make an assert macro that prints out byte slices as bytes when it fails.
    use super::*;

    #[test]
    fn jump() {
        assert_eq!(
            node(b"JUMP L0  h"),
            Ok((&b"  h"[..], IrNode::Jump(Label(b"L0".into()))))
        );
        assert_eq!(
            node(b"JUMP alskdhjfa"),
            Ok((&b""[..], IrNode::Jump(Label(b"alskdhjfa".into()))))
        );
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
    fn iconst_sconst() {
        assert_eq!(node(b"ICONST 50"), Ok((&b""[..], IrNode::Iconst(50))));
        // Here is where I deviate from the format as produced by the printer in ir.h, but all I'm doing is adding one escape sequence to strings: \n, for newline.
        assert_eq!(
            node(br#"SCONST Hello"#),
            Ok((&b""[..], IrNode::Sconst("Hello".into())))
        );
        assert_eq!(
            node(br#"SCONST Hello\n"#),
            // TODO: Expect the \n to be transformed away.
            Ok((&b""[..], IrNode::Sconst("Hello\\n".into())))
        );
    }

    #[test]
    fn simple_program() {
        assert_eq!(program(b"band"), Ok(vec![IrNode::Band]));
        assert_eq!(
            program(
                b"band\n\
                     bor\n\
                     and\n\
                     xor"
            ), // Works without terminating newline.
            Ok(vec![IrNode::Band, IrNode::Bor, IrNode::And, IrNode::Xor,])
        );
        assert_eq!(
            program(
                b"band\n\
                     BOR\n\
                     And\n\
                     xOR\n"
            ), // Also works with terminating newline.
            Ok(vec![IrNode::Band, IrNode::Bor, IrNode::And, IrNode::Xor,])
        );

        // Other whitespace:
        assert_eq!(
            program(
                b" band \n \
                     BOR\n \t\
                     And \n     \
                     \txOR \n"
            ), // Also works with terminating newline.
            Ok(vec![IrNode::Band, IrNode::Bor, IrNode::And, IrNode::Xor,])
        );
    }

    #[test]
    fn slightly_more_complex_program() {
        assert_eq!(
            program(
                b"Iconst 500\n\
                  Iconst 0"
            ),
            Ok(vec![IrNode::Iconst(500), IrNode::Iconst(0),])
        )
    }
}

// TODO: Each instruction function should not take trailing whitespace. That should be left to the thing that processes multiple instructions, that can take newlines and spaces. I think instructions could totally legally be on the same line!
// For now, I'm actually requiring a newline between them, but I don't know if I need to.
