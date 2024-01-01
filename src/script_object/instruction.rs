use std::{str::FromStr, string::ParseError};

use super::value_box;

type VBMA = value_box::ValueBoxMemAddress;
type Anchor = String;

pub enum Instruction {
    // I/O
    In,
    Out,

    // COPY
    CopyFrom(VBMA),
    CopyTo(VBMA),

    // MATH
    Add(VBMA),
    Sub(VBMA),
    BumpUp(VBMA),
    BumpDown(VBMA),

    // JUMP
    Jump(Anchor),
    JumpIfZero(Anchor),
    JumpIfNegative(Anchor),
}

impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
    }
}
