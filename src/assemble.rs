use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag_no_case, take_while},
    character::complete::{
        char as nom_char, i64 as nom_i64, multispace0, multispace1, none_of, space1,
    },
    combinator::{all_consuming, value},
    multi::separated_list0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::ir_definition::{IrNode, Label};
type NodeResult<'a> = IResult<&'a str, IrNode>;

fn identifier(input: &str) -> IResult<&str, &str> {
    // TODO: Make this require there to be at least one thing in the input.
    take_while(|c| char::is_alphanumeric(c) || c == '$' || c == '_')(input)
}

fn iconst(input: &str) -> NodeResult {
    let (rest, i) = 
        preceded(tuple((tag_no_case("ICONST"), space1)), nom_i64)(input)?;
    Ok((rest, IrNode::Iconst(i)))
}

fn string_literal(input: &str) -> IResult<&str, String> {
    use nom::bytes::complete::tag;
    delimited(
        nom_char('\"'),
        escaped_transform(
            none_of("\\\""),
            '\\',
            alt((value("\\", tag("\\")), value("\"", tag("\"")))),
        ),
        nom_char('\"'),
    )(input)
}

fn sconst(input: &str) -> NodeResult {
    let (rest, transformed_text) = preceded(tuple((tag_no_case("SCONST"), space1)), string_literal)(input)?;
    Ok((rest, IrNode::Sconst(transformed_text.into())))
}

// No-arg nodes:
// TODO: These should be done through a macro, but I don't know how to macro right now.
// Could also be a function that returns a function, but when I tried to write that it had to copy the IrNode.
fn nop(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("NOP")(input)?;
    Ok((rest, IrNode::Nop))
}

fn add(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("ADD")(input)?;
    Ok((rest, IrNode::Add))
}

fn sub(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("SUB")(input)?;
    Ok((rest, IrNode::Sub))
}

fn mul(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("MUL")(input)?;
    Ok((rest, IrNode::Mul))
}

fn div(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("DIV")(input)?;
    Ok((rest, IrNode::Div))
}

fn mod_(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("MOD")(input)?;
    Ok((rest, IrNode::Mod))
}

fn bor(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("BOR")(input)?;
    Ok((rest, IrNode::Bor))
}

fn band(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("BAND")(input)?;
    Ok((rest, IrNode::Band))
}

fn xor(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("XOR")(input)?;
    Ok((rest, IrNode::Xor))
}

fn or(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("OR")(input)?;
    Ok((rest, IrNode::Or))
}

fn and(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("AND")(input)?;
    Ok((rest, IrNode::And))
}

fn eq(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("EQ")(input)?;
    Ok((rest, IrNode::Eq))
}

fn lt(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("LT")(input)?;
    Ok((rest, IrNode::Lt))
}

fn gt(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("GT")(input)?;
    Ok((rest, IrNode::Gt))
}

fn not(input: &str) -> NodeResult {
    let (rest, _) = tag_no_case("NOT")(input)?;
    Ok((rest, IrNode::Not))
}

fn jump(input: &str) -> NodeResult {
    let (rest, label_text) = preceded(tuple((tag_no_case("JUMP"), space1)), identifier)(input)?;
    Ok((rest, IrNode::Jump(Label(label_text.into()))))
}

pub fn node(input: &str) -> NodeResult {
    alt((
        iconst, sconst, nop, add, sub, mul, div, mod_, bor, band, xor, or, and, eq, lt, gt, not,
        jump,
    ))(input)
}

pub fn program(input: &str) -> Result<Vec<IrNode>, nom::Err<nom::error::Error<&str>>> {
    // TODO: Handle the final missing newline. This somehow doesn't work.
    let (rest, prog) = all_consuming(delimited(
        multispace0,
        separated_list0(multispace1, node),
        multispace0,
    ))(input)?;
    assert_eq!(rest, ""); // Surely this is redundant because of how all-consuming works.
    Ok(prog)
}

#[cfg(test)]
mod tests {
    // TODO: Make an assert macro that prints out byte slices as bytes when it fails.
    use super::*;

    #[test]
    fn jump() {
        assert_eq!(
            node("JUMP L0  h"),
            Ok(("  h", IrNode::Jump(Label("L0".into()))))
        );
        assert_eq!(
            node("JUMP alskdhjfa"),
            Ok(("", IrNode::Jump(Label("alskdhjfa".into()))))
        );
    }

    #[test]
    fn noarg_nodes() {
        // I never know how many tests to write...
        // Positive examples:
        assert_eq!(node("ADD "), Ok((" ", IrNode::Add)));
        assert_eq!(node("NOP"), Ok(("", IrNode::Nop)));
        assert_eq!(node("sUb   kdf"), Ok(("   kdf", IrNode::Sub)));
        assert_eq!(node("Mul "), Ok((" ", IrNode::Mul)));
        assert_eq!(node("diV  "), Ok(("  ", IrNode::Div)));
        assert_eq!(node("mod  $$04"), Ok(("  $$04", IrNode::Mod)));
        assert_eq!(node("BOR      \n"), Ok(("      \n", IrNode::Bor)));
        assert_eq!(node("bANd  "), Ok(("  ", IrNode::Band)));
        assert_eq!(node("xor"), Ok(("", IrNode::Xor)));
        assert_eq!(node("or"), Ok(("", IrNode::Or)));
        assert_eq!(node("and"), Ok(("", IrNode::And)));
        assert_eq!(node("eq"), Ok(("", IrNode::Eq)));
        assert_eq!(node("lT"), Ok(("", IrNode::Lt)));
        assert_eq!(node("gt"), Ok(("", IrNode::Gt)));
        assert_eq!(node("Not"), Ok(("", IrNode::Not)));

        // Negative examples:
        assert!(node("n ot").is_err());
        assert!(node(" div").is_err()); // Doesn't accept leading whitespace.
        assert_ne!(node("bor   "), Ok(("", IrNode::Bor))); // Doesn't accept trailing whitespace.
    }

    #[test]
    fn iconst_sconst() {
        assert_eq!(node("ICONST 50"), Ok(("", IrNode::Iconst(50))));
        // Here is where I deviate from the format as produced by the printer in ir.h, but all I'm doing is adding one escape sequence to strings: \n, for newline.
        assert_eq!(
            node("SCONST \"Hello\""),
            Ok(("", IrNode::Sconst("Hello".into())))
        );
        assert_eq!(
            node("SCONST \"Hello\"\n"),
            Ok(("\n", IrNode::Sconst("Hello".into())))
        );

        assert_eq!(
            node("SCONST \"Hello I'm a string with no escapes\""),
            Ok((
                "",
                IrNode::Sconst("Hello I'm a string with no escapes".into())
            ))
        );

        assert_eq!(
            node("SCONST \" with \n newlines\n\""),
            Ok(("", IrNode::Sconst(" with \n newlines\n".into())))
        );

        assert_eq!(
            node("sConst \" with \\\" literal quotes \\\" \""),
            Ok(("", IrNode::Sconst(" with \" literal quotes \" ".into())))
        );

        assert_eq!(
            node("SCONST \" \t with tabs and literal \\\\ backslashes\""),
            Ok((
                "",
                IrNode::Sconst(" \t with tabs and literal \\ backslashes".into())
            ))
        );
    }

    #[test]
    fn simple_program() {
        assert_eq!(program("band"), Ok(vec![IrNode::Band]));
        assert_eq!(
            program(
                "band\n\
                bor\n\
                and\n\
                xor"
            ), // Works without terminating newline.
            Ok(vec![IrNode::Band, IrNode::Bor, IrNode::And, IrNode::Xor,])
        );
        assert_eq!(
            program(
                "band\n\
                     BOR\n\
                     And\n\
                     xOR\n"
            ), // Also works with terminating newline.
            Ok(vec![IrNode::Band, IrNode::Bor, IrNode::And, IrNode::Xor,])
        );

        // Other whitespace:
        assert_eq!(
            program(
                " band \n \
                     BOR\n \t\
                     And \n     \
                     \txOR \n"
            ), // Also works with terminating newline.
            Ok(vec![IrNode::Band, IrNode::Bor, IrNode::And, IrNode::Xor,])
        );
    }

    #[test]
    fn slightly_more_complex_programs() {
        assert_eq!(
            program(
                "Iconst 500\n\
                 Iconst 0"
            ),
            Ok(vec![IrNode::Iconst(500), IrNode::Iconst(0),])
        );

        assert_eq!(
            program(
                "Sconst \"Hello I'm a string with no escapes\"\n\
                 Sconst \"with double quotes \\\" \"\n\
                 Sconst    \"with \n newlines \n\" \n\
                 Sconst \"\\\\ with backslashes \\\\\" \n\
                 Iconst 20"
            ),
            Ok(vec![
                IrNode::Sconst("Hello I'm a string with no escapes".into()),
                IrNode::Sconst("with double quotes \" ".into()),
                IrNode::Sconst("with \n newlines \n".into()),
                IrNode::Sconst("\\ with backslashes \\".into()),
                IrNode::Iconst(20),
            ])
        );
    }
}

// Each instruction function should not take trailing whitespace. That should be left to the thing that processes multiple instructions, that can take newlines and spaces.
