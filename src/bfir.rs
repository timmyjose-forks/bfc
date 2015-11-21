
use std::fmt;
use std::num::Wrapping;
use std::collections::HashMap;

use diagnostics::{Info,Level};

use self::Instruction::*;

pub type Cell = Wrapping<i8>;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Instruction {
    Increment {
        amount: Cell,
        offset: isize,
    },
    PointerIncrement(isize),
    Read,
    Write,
    Loop(Vec<Instruction>),
    // These instruction have no direct equivalent in BF, but we
    // generate them during optimisation.
    Set {
        amount: Cell,
        offset: isize,
    },
    MultiplyMove(HashMap<isize, Cell>),
}

fn fmt_with_indent(instr: &Instruction, indent: i32, f: &mut fmt::Formatter) {
    for _ in 0..indent {
        let _ = write!(f, "  ");
    }

    match instr {
        &Loop(ref loop_body) => {
            let _ = write!(f, "Loop");

            for loop_instr in loop_body {
                let _ = write!(f, "\n");
                fmt_with_indent(loop_instr, indent + 1, f);
            }
        }
        instr => {
            let _ = write!(f, "{:?}", instr);
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_with_indent(self, 0, f);
        Ok(())
    }
}

/// Given a string of BF source code, parse and return our BF IR
/// representation. If parsing fails, return an Info describing what
/// went wrong.
pub fn parse(filename: &str, source: &str) -> Result<Vec<Instruction>, Info> {
    // Instructions in the current loop (or toplevel).
    let mut instructions = vec![];
    // Contains the instructions of open parent loops (or toplevel),
    // and the starting indices of the loops.
    let mut stack = vec![];


    for (index, c) in source.chars().enumerate() {
        match c {
            '+' => instructions.push(Increment {
                amount: Wrapping(1),
                offset: 0,
            }),
            '-' => instructions.push(Increment {
                amount: Wrapping(-1),
                offset: 0,
            }),
            '>' => instructions.push(PointerIncrement(1)),
            '<' => instructions.push(PointerIncrement(-1)),
            ',' => instructions.push(Read),
            '.' => instructions.push(Write),
            '[' => {
                stack.push((instructions, index));
                instructions = vec![];
            }
            ']' => {
                if let Some((mut parent_instr, _)) = stack.pop() {
                    parent_instr.push(Loop(instructions));
                    instructions = parent_instr;
                } else {
                    return Err(Info {
                        level: Level::Error,
                        filename: filename.to_owned(),
                        message: "This ] has no matching [".to_owned(),
                        position: Some(index..index),
                        source: Some(source.to_owned()),
                    })
                }
            }
            _ => (),
        }
    }

    if !stack.is_empty() {
        let pos = stack.last().unwrap().1;
        return Err(Info {
            level: Level::Error,
            filename: filename.to_owned(),
            message: "This [ has no matching ]".to_owned(),
            position: Some(pos..pos),
            source: Some(source.to_owned())
        })
    }

    Ok(instructions)
}

#[test]
fn parse_increment() {
    assert_eq!(parse("", "+").unwrap(),
               [Increment {
                   amount: Wrapping(1),
                   offset: 0,
               }]);
    assert_eq!(parse("", "++").unwrap(),
               [Increment {
                   amount: Wrapping(1),
                   offset: 0,
               },
                Increment {
                   amount: Wrapping(1),
                   offset: 0,
               }]);
}

#[test]
fn parse_decrement() {
    assert_eq!(parse("", "-").unwrap(),
               [Increment {
                   amount: Wrapping(-1),
                   offset: 0,
               }]);
}

#[test]
fn parse_pointer_increment() {
    assert_eq!(parse("", ">").unwrap(), [PointerIncrement(1)]);
}

#[test]
fn parse_pointer_decrement() {
    assert_eq!(parse("", "<").unwrap(), [PointerIncrement(-1)]);
}

#[test]
fn parse_read() {
    assert_eq!(parse("", ",").unwrap(), [Read]);
}

#[test]
fn parse_write() {
    assert_eq!(parse("", ".").unwrap(), [Write]);
}

#[test]
fn parse_empty_loop() {
    let expected = [Loop(vec![])];
    assert_eq!(parse("", "[]").unwrap(), expected);
}

#[test]
fn parse_simple_loop() {
    let loop_body = vec![Increment {
        amount: Wrapping(1),
        offset: 0,
    }];
    let expected = [Loop(loop_body)];
    assert_eq!(parse("", "[+]").unwrap(), expected);
}

#[test]
fn parse_complex_loop() {
    let loop_body = vec![Read,
                         Increment {
        amount: Wrapping(1),
        offset: 0,
    }];
    let expected = [Write,
                    Loop(loop_body),
                    Increment {
        amount: Wrapping(-1),
        offset: 0,
    }];
    assert_eq!(parse("", ".[,+]-").unwrap(), expected);
}

#[test]
fn parse_unbalanced_loop() {
    assert!(parse("", "[").is_err());
    assert!(parse("", "]").is_err());
    assert!(parse("", "][").is_err());
    assert!(parse("", "[][").is_err());
}

#[test]
fn parse_comment() {
    assert_eq!(parse("", "foo! ").unwrap(), []);
}
