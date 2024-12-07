use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag_no_case, take_till, take_while1},
    character::complete::{char as nom_char, i64 as nom_i64, none_of, u64 as nom_u64},
    combinator::{all_consuming, map, opt, value},
    multi::{many0_count, many1_count, separated_list0},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use crate::ir_definition::{Intrinsic, IrNode, Label};
type NodeResult<'a> = IResult<&'a str, IrNode>;

fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c| char::is_alphanumeric(c) || c == '$' || c == '_')(input)
}

fn inside_string(input: &str) -> IResult<&str, String> {
    use nom::bytes::complete::tag;
    // STRETCH: Okay, there's gotta be a better way. Why do I need to use opt
    // for this to work correctly within string_literal?
    map(
        opt(escaped_transform(
            none_of(r#"\""#),
            '\\',
            alt((value(r"\", tag(r"\")), value(r#"""#, tag(r#"""#)))),
        )),
        |inner_text| inner_text.unwrap_or("".into()),
    )(input)
}

fn string_literal(input: &str) -> IResult<&str, String> {
    delimited(nom_char('"'), inside_string, nom_char('"'))(input)
}

fn multi_line_comment(input: &str) -> IResult<&str, &str> {
    use nom::bytes::complete::{tag, take_until};
    delimited(tag("/*"), take_until("*/"), tag("*/"))(input)
}

// Does not consume the thing that ended the single_line_comment (either a newline or the end of the file).
fn single_line_comment(input: &str) -> IResult<&str, &str> {
    use nom::bytes::complete::tag;

    // TODO: Try making this use `terminated`, `line_ending`, and `eof`.
    preceded(tag("#"), take_till(|c| c == '\n' || c == '\r'))(input)
}

fn within_node(input: &str) -> IResult<&str, &str> {
    use nom::{character::complete::space1, combinator::recognize};
    recognize(many0_count(alt((space1, multi_line_comment))))(input)
}

fn between_nodes(input: &str) -> IResult<&str, &str> {
    use nom::{character::complete::multispace1, combinator::recognize};
    recognize(many1_count(alt((
        multispace1,
        multi_line_comment,
        single_line_comment,
    ))))(input)
}

macro_rules! noarg_node {
    ($func_name:ident, $tag_text:literal, $result:expr) => {
        fn $func_name(input: &str) -> NodeResult {
            let (rest, _) = tag_no_case($tag_text)(input)?;
            Ok((rest, $result))
        }
    };
}

// Each instruction function should not take trailing whitespace. That should be
// left to the thing that processes multiple instructions, that can take
// newlines and spaces.

fn iconst(input: &str) -> NodeResult {
    let (rest, i) = preceded(tuple((tag_no_case("ICONST"), within_node)), nom_i64)(input)?;
    Ok((rest, IrNode::Iconst(i)))
}

fn sconst(input: &str) -> NodeResult {
    let (rest, transformed_text) =
        preceded(tuple((tag_no_case("SCONST"), within_node)), string_literal)(input)?;
    Ok((rest, IrNode::Sconst(transformed_text.into())))
}

noarg_node!(nop, "NOP", IrNode::Nop);
noarg_node!(add, "ADD", IrNode::Add);
noarg_node!(sub, "SUB", IrNode::Sub);
noarg_node!(mul, "MUL", IrNode::Mul);
noarg_node!(div, "DIV", IrNode::Div);
noarg_node!(mod_, "MOD", IrNode::Mod);
noarg_node!(bor, "BOR", IrNode::Bor);
noarg_node!(band, "BAND", IrNode::Band);
noarg_node!(xor, "XOR", IrNode::Xor);
noarg_node!(or, "OR", IrNode::Or);
noarg_node!(and, "AND", IrNode::And);
noarg_node!(eq, "EQ", IrNode::Eq);
noarg_node!(lt, "LT", IrNode::Lt);
noarg_node!(gt, "GT", IrNode::Gt);
noarg_node!(not, "NOT", IrNode::Not);

fn reserve(input: &str) -> NodeResult {
    let (start_of_string_or_null, (name, size)) = preceded(
        tag_no_case("RESERVE"),
        tuple((
            preceded(within_node, identifier),
            // Is there every a good reason to reserve a negative amount of space?
            delimited(within_node, nom_u64, within_node),
        )),
    )(input)?;

    if start_of_string_or_null.as_bytes()[0] == b'\"' {
        let (rest, initial_value) = string_literal(start_of_string_or_null)?;
        return Ok((
            rest,
            IrNode::ReserveString {
                size,
                name: name.into(),
                initial_value,
            },
        ));
    } else {
        let (rest, _) = tag_no_case("(null)")(start_of_string_or_null)?;
        return Ok((rest, IrNode::ReserveInt { name: name.into() }));
    }
}

fn read(input: &str) -> NodeResult {
    let (rest, name) = preceded(tuple((tag_no_case("READ"), within_node)), identifier)(input)?;
    Ok((rest, IrNode::Read(name.into())))
}

fn write(input: &str) -> NodeResult {
    let (rest, name) = preceded(tuple((tag_no_case("WRITE"), within_node)), identifier)(input)?;
    Ok((rest, IrNode::Write(name.into())))
}

fn arg_local_read(input: &str) -> NodeResult {
    let (rest, index) =
        preceded(tuple((tag_no_case("ARGLOCAL_READ"), within_node)), nom_u64)(input)?;
    Ok((rest, IrNode::ArgLocalRead(index)))
}

fn arg_local_write(input: &str) -> NodeResult {
    let (rest, index) =
        preceded(tuple((tag_no_case("ARGLOCAL_WRITE"), within_node)), nom_u64)(input)?;
    Ok((rest, IrNode::ArgLocalWrite(index)))
}

fn label(input: &str) -> NodeResult {
    let (rest, name) = terminated(identifier, tag_no_case(":"))(input)?;
    Ok((rest, IrNode::Label(Label::named(name))))
}

fn jump(input: &str) -> NodeResult {
    let (rest, name) = preceded(tuple((tag_no_case("JUMP"), within_node)), identifier)(input)?;
    Ok((rest, IrNode::Jump(Label::named(name))))
}

fn branch_zero(input: &str) -> NodeResult {
    let (rest, name) =
        preceded(tuple((tag_no_case("BRANCHZERO"), within_node)), identifier)(input)?;
    Ok((rest, IrNode::BranchZero(Label::named(name))))
}

fn function(input: &str) -> NodeResult {
    let (rest, (name, num_locs)) = preceded(
        tuple((tag_no_case("FUNCTION"), within_node)),
        tuple((identifier, preceded(within_node, nom_u64))),
    )(input)?;
    Ok((
        rest,
        IrNode::Function {
            label: Label::named(name),
            num_locs,
        },
    ))
}

fn call(input: &str) -> NodeResult {
    let (rest, (name, num_args)) = preceded(
        tuple((tag_no_case("CALL"), within_node)),
        tuple((identifier, preceded(within_node, nom_u64))),
    )(input)?;
    Ok((
        rest,
        IrNode::Call {
            label: Label::named(name),
            num_args,
        },
    ))
}

noarg_node!(ret, "RET", IrNode::Ret);

fn intrinsic(input: &str) -> NodeResult {
    let (rest, intrinsic) = preceded(
        tuple((tag_no_case("INTRINSIC"), within_node)),
        alt((
            value(Intrinsic::PrintInt, tag_no_case("PRINT_INT")),
            value(Intrinsic::PrintString, tag_no_case("PRINT_STRING")),
            value(Intrinsic::Exit, tag_no_case("EXIT")),
        )),
    )(input)?;

    Ok((rest, IrNode::Intrinsic(intrinsic)))
}

fn push(input: &str) -> NodeResult {
    let (rest, reg) = preceded(tuple((tag_no_case("PUSH"), within_node)), nom_i64)(input)?;
    Ok((rest, IrNode::Push { reg }))
}

fn pop(input: &str) -> NodeResult {
    let (rest, reg) = preceded(tuple((tag_no_case("POP"), within_node)), nom_i64)(input)?;
    Ok((rest, IrNode::Pop { reg }))
}

pub fn node(input: &str) -> NodeResult {
    alt((
        alt((
            iconst, sconst, nop, add, sub, mul, div, mod_, bor, band, xor, or, and, eq, lt, gt, not,
        )),
        alt((reserve, read, write, arg_local_read, arg_local_write)),
        alt((label, jump, branch_zero)),
        alt((function, call, ret, intrinsic)),
        alt((push, pop)),
    ))(input)
}

pub fn program(input: &str) -> Result<Vec<IrNode>, nom::Err<nom::error::Error<&str>>> {
    // TODO: Try doing this more simply. Do I need to consider the separators differently from the starting and ending whitespace?
    let (rest, prog) = all_consuming(delimited(
        opt(between_nodes),
        separated_list0(between_nodes, node),
        opt(between_nodes),
    ))(input)?;
    assert_eq!(rest, ""); // Surely this is redundant because of how all-consuming works.
    Ok(prog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inside_string_test() {
        assert_eq!(inside_string(""), Ok(("", "".into())));
        assert_eq!(inside_string(r#"\""#), Ok(("", r#"""#.into())));
        assert_eq!(
            inside_string(r#"I have some literal quotes \" \"."#),
            Ok(("", r#"I have some literal quotes " "."#.into()))
        );
        assert_eq!(
            inside_string(r"I \\ have some \\ literal \\\\ backslashes."),
            Ok(("", r"I \ have some \ literal \\ backslashes.".into()))
        );
        assert_eq!(
            inside_string(r#"Some \\ \" \"\" literal backslashes\\\\ and quotes."#),
            Ok((
                "",
                r#"Some \ " "" literal backslashes\\ and quotes."#.into()
            ))
        );

        assert_eq!(
            inside_string(r#"I don't include the unescaped quote.""#),
            Ok((r#"""#, "I don't include the unescaped quote.".into()))
        );
        assert_eq!(
            inside_string(r#"I don't get matched because I have an invalid escape sequence: \n "#),
            Ok((
                r#"I don't get matched because I have an invalid escape sequence: \n "#,
                "".into()
            ))
        );
        assert_eq!(
            inside_string(r#"I don't get matched because I end in a backslash \"#),
            Ok((
                r#"I don't get matched because I end in a backslash \"#,
                "".into()
            ))
        );
    }

    #[test]
    fn string_literal_test() {
        // TODO: Add more tests.
        assert_eq!(string_literal(r#"" ""#), Ok(("", " ".into())));
        assert_eq!(
            string_literal(r#""I don't include the unescaped quote.""#),
            Ok(("", "I don't include the unescaped quote.".into()))
        );
        assert_eq!(string_literal(r#""""#), Ok(("", "".into())));
        assert_eq!(string_literal(r#""\"""#), Ok(("", r#"""#.into())));
        assert_eq!(
            string_literal(r#""\"Around and around, good fun\"""#),
            Ok(("", r#""Around and around, good fun""#.into()))
        );
    }

    #[test]
    fn identifier_test() {
        assert_eq!(identifier("hello"), Ok(("", "hello")));
        assert_eq!(identifier("$bruh"), Ok(("", "$bruh")));
        assert_eq!(identifier("_bruh"), Ok(("", "_bruh")));
        assert_eq!(identifier("123br21"), Ok(("", "123br21")));
        assert_eq!(identifier("hello goodbye"), Ok((" goodbye", "hello")));

        assert!(identifier("").is_err());
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

        assert_eq!(node("SCONST \"\""), Ok(("", IrNode::Sconst("".into()))));

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
    fn reserve() {
        // STRETCH: Should I let the user know when they're reserving the wrong amount of space for strings?
        // Reserving strings:
        assert_eq!(
            node("Reserve var 10 \"Hello world\""),
            Ok((
                "",
                IrNode::ReserveString {
                    size: 10,
                    name: "var".into(),
                    initial_value: "Hello world".into()
                }
            ))
        );

        assert_eq!(
            node("Reserve 1bruh1 20 \"I \\\\ have a bunch \n \\\" of weird stuff\"  "),
            Ok((
                "  ",
                IrNode::ReserveString {
                    size: 20,
                    name: "1bruh1".into(),
                    initial_value: "I \\ have a bunch \n \" of weird stuff".into()
                }
            ))
        );

        // Reserving integers:
        assert_eq!(
            node("Reserve $$FREAKY_INTERNAL_COMPILER_GLOBAL$$ 4 (null)\t\n"),
            Ok((
                "\t\n",
                IrNode::ReserveInt {
                    name: "$$FREAKY_INTERNAL_COMPILER_GLOBAL$$".into()
                }
            ))
        );

        assert_eq!(
            node("RESERVE $_$ 4 (null)"),
            Ok(("", IrNode::ReserveInt { name: "$_$".into() }))
        )
    }

    #[test]
    fn variables() {
        // Globals:
        assert_eq!(node("WRITE $$$"), Ok(("", IrNode::Write("$$$".into()))));

        assert_eq!(node("READ _lkas"), Ok(("", IrNode::Read("_lkas".into()))));

        assert_eq!(
            node("read kddk\n \t"),
            Ok(("\n \t", IrNode::Read("kddk".into())))
        );

        // Locals:
        assert_eq!(
            node("ARGLOCAL_READ 200"),
            Ok(("", IrNode::ArgLocalRead(200)))
        );

        assert_eq!(
            node("ARGLOCAL_WRITE  \t 40"),
            Ok(("", IrNode::ArgLocalWrite(40)))
        );

        // Negative locals are not allowed:
        assert!(node("ARGLOCAL_READ -1").is_err());
        assert!(node("ARGLOCAL_WRITE -2340").is_err());

        // Instructions on locals do not take names:
        assert!(node("ARGLOCAL_READ illegal_named_local").is_err());
    }

    #[test]
    fn control_flow() {
        // Label:
        assert_eq!(
            node("birch:"),
            Ok(("", IrNode::Label(Label::named("birch"))))
        );

        assert_eq!(node("Sam:"), Ok(("", IrNode::Label(Label::named("Sam")))));

        // Jump:
        assert_eq!(
            node("JUMP L0  h"),
            Ok(("  h", IrNode::Jump(Label::named("L0"))))
        );
        assert_eq!(
            node("JUMP alskdhjfa"),
            Ok(("", IrNode::Jump(Label::named("alskdhjfa"))))
        );

        // BranchZero:
        assert_eq!(
            node("branchzero l20"),
            Ok(("", IrNode::BranchZero(Label::named("l20"))))
        );
        assert_eq!(
            node("branchZERO foo\n"),
            Ok(("\n", IrNode::BranchZero(Label::named("foo"))))
        );
    }

    #[test]
    fn functions() {
        // Function:

        assert_eq!(
            node("FuncTion no_locals 0"),
            Ok((
                "",
                IrNode::Function {
                    label: Label::named("no_locals"),
                    num_locs: 0
                }
            ))
        );

        assert_eq!(
            node("FUNCTION some_locals 3"),
            Ok((
                "",
                IrNode::Function {
                    label: Label::named("some_locals"),
                    num_locs: 3
                }
            ))
        );

        assert!(node("function negative_locs -5050").is_err());
        assert!(node("function locs_not_specified ").is_err());

        // Call:

        assert_eq!(
            node("CALL no_args 0\t\tbruh"),
            Ok((
                "\t\tbruh",
                IrNode::Call {
                    label: Label::named("no_args"),
                    num_args: 0
                }
            ))
        );

        assert_eq!(
            node("CALL many_args 6"),
            Ok((
                "",
                IrNode::Call {
                    label: Label::named("many_args"),
                    num_args: 6
                }
            ))
        );

        assert!(node("caLL negative_args -5").is_err());
        assert!(node("CALL args_not_specified\t").is_err());

        // Ret:

        assert_eq!(node("ret"), Ok(("", IrNode::Ret)));
        assert_eq!(node("return"), Ok(("urn", IrNode::Ret))); // Tough luck. Keep your english words away from me!

        // Intrinsic:

        assert_eq!(
            node("Intrinsic PRINT_STRING"),
            Ok(("", IrNode::Intrinsic(Intrinsic::PrintString)))
        );
        assert_eq!(
            node("INTRINSIC print_int"),
            Ok(("", IrNode::Intrinsic(Intrinsic::PrintInt)))
        );
        assert_eq!(
            node("Intrinsic exit"),
            Ok(("", IrNode::Intrinsic(Intrinsic::Exit)))
        );

        assert!(node("intrinsic not_an_intrinsic").is_err());

        assert!(node("intrinsic").is_err()); // Intrinsic not specified.
    }

    #[test]
    fn push_pop() {
        // Push:
        assert_eq!(node("Push 1"), Ok(("", IrNode::Push { reg: 1 })));
        assert_eq!(node("PUSH 2020"), Ok(("", IrNode::Push { reg: 2020 })));

        assert!(node("PUSH").is_err()); // Bare push not allowed

        // Pop:
        assert_eq!(node("pop -1"), Ok(("", IrNode::Pop { reg: -1 })));
        assert_eq!(node("poP 2013 "), Ok((" ", IrNode::Pop { reg: 2013 })));

        assert!(node("POP").is_err()); // Bare pop also not allowed...
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

    #[test]
    fn single_line_comment_test() {
        assert!(single_line_comment("").is_err()); // The empty string is not a single-line comment.

        // Single line comments must start precisely with a hash, but there can
        // be space(s) between the end of a node and a single-line comment
        // handled by `between_nodes`:
        assert!(single_line_comment(" #").is_err());
        assert!(single_line_comment("//").is_err()); // Sorry, C fans.
        assert!(single_line_comment("input").is_err());

        assert_eq!(single_line_comment("#"), Ok(("", ""))); // The # is not part of the result of the comment.
        assert_eq!(single_line_comment("#  Hello"), Ok(("", "  Hello")));
        assert_eq!(single_line_comment("# Hello\n"), Ok(("\n", " Hello"))); // Single-line comments end before the first newline.
        assert_eq!(
            single_line_comment("# First single-line comment\n # Second single line comment"),
            Ok((
                "\n # Second single line comment",
                " First single-line comment"
            ))
        );
        assert_eq!(
            single_line_comment(
                "# ;laisupowielkjbo982349867q345\\ \n Oh boy this is not part of that comment"
            ),
            Ok((
                "\n Oh boy this is not part of that comment",
                " ;laisupowielkjbo982349867q345\\ "
            ))
        );
    }

    #[test]
    fn multi_line_comment_test() {
        assert!(multi_line_comment("").is_err()); // Empty string is not a multi-line comment.
        assert!(multi_line_comment("/*").is_err()); // Multi-line comments must be terminated.

        assert_eq!(multi_line_comment("/**/"), Ok(("", ""))); // The delimiters are not part of the result of the comment.
        assert_eq!(multi_line_comment("/* */"), Ok(("", " ")));
        assert_eq!(
            multi_line_comment("/* Hello I can be anything !! sconst!*/"),
            Ok(("", " Hello I can be anything !! sconst!"))
        );
        assert_eq!(
            multi_line_comment("/* SCONST ICONST */"),
            Ok(("", " SCONST ICONST "))
        );
        assert_eq!(multi_line_comment("/* Jump */  "), Ok(("  ", " Jump ")));

        assert_eq!(multi_line_comment("/* */ */"), Ok((" */", " "))); // Multi-line comments end at the first ending delimiter.
        assert_eq!(
            multi_line_comment("/* \n\n \\n \\\" */"),
            Ok(("", " \n\n \\n \\\" "))
        ); // Nothing is special in a multi-line comment.
    }

    #[test]
    fn programs_with_single_line_comments() {
        assert_eq!(
            program(
                r##"Sconst "Have a string, why don'tcha "
                Iconst -30 # Very important comment
                L0: sconst "\"Around and around, good fun\"" # Just like malloc! 
                JUMP L0 
                # This next bit is incredibly confusing, but must not be changed!!!
                # TODO: Fix.
                BRANCHZERO L1
                L1:
                "##
            ),
            Ok(vec![
                IrNode::Sconst("Have a string, why don'tcha ".into()),
                IrNode::Iconst(-30),
                IrNode::Label(Label::named("L0")),
                IrNode::Sconst("\"Around and around, good fun\"".into()),
                IrNode::Jump(Label::named("L0")),
                IrNode::BranchZero(Label::named("L1")),
                IrNode::Label(Label::named("L1")),
            ])
        );
    }

    #[test]
    fn programs_with_any_kind_of_comment() {
        assert_eq!(
            program(
                "Iconst 40\n\
                 Jump L1\n\
                 # Single line comment on it's own line.\n\
                 \n\
                 /* Multi-line comment on one line. */\n\
                 \n\
                 \n\
                 /* Multi-line comment spanning\n\
                    two lines. */\n\
                 Iconst 20\n\
                 Iconst 40\n\
                 Add\n\
                 Intrinsic print_int\n\
                 Intrinsic exit"
            ),
            Ok(vec![
                IrNode::Iconst(40),
                IrNode::Jump(Label::named("L1")),
                IrNode::Iconst(20),
                IrNode::Iconst(40),
                IrNode::Add,
                IrNode::Intrinsic(Intrinsic::PrintInt),
                IrNode::Intrinsic(Intrinsic::Exit)
            ])
        );
    }
}
