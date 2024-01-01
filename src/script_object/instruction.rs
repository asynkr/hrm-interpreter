use std::{str::FromStr, error::Error};
use collapse::collapse;

use super::value_box;

type VBMA = value_box::ValueBoxMemAddress;
type Anchor = String;

#[derive(Debug, PartialEq)]
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
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        let s: &str = &collapse(s);
        let parts = s.split_whitespace().collect::<Vec<&str>>();

        if parts.len() > 2 {
            return Err("Instruction line must have at most two parts separated by white spaces".into());
        }

        let instruction_key = *parts.get(0).unwrap();
        let address_key = parts.get(1).cloned();

        match (instruction_key, address_key) {
            ("INBOX",       None) => Ok(Instruction::In),
            ("OUTBOX",      None) => Ok(Instruction::Out),
            ("COPYFROM",    Some(akey)) => Ok(Instruction::CopyFrom(VBMA::from_str(akey)?)),
            ("COPYTO",      Some(akey)) => Ok(Instruction::CopyTo(VBMA::from_str(akey)?)),
            ("ADD",         Some(akey)) => Ok(Instruction::Add(VBMA::from_str(akey)?)),
            ("SUB",         Some(akey)) => Ok(Instruction::Sub(VBMA::from_str(akey)?)),
            ("BUMPUP",      Some(akey)) => Ok(Instruction::BumpUp(VBMA::from_str(akey)?)),
            ("BUMPDN",      Some(akey)) => Ok(Instruction::BumpDown(VBMA::from_str(akey)?)),
            ("JUMP",        Some(akey)) => Ok(Instruction::Jump(akey.to_string())),
            ("JUMPZ",       Some(akey)) => Ok(Instruction::JumpIfZero(akey.to_string())),
            ("JUMPN",       Some(akey)) => Ok(Instruction::JumpIfNegative(akey.to_string())),
            _ => Err("Invalid instruction".into()),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_from_str() {
        assert_eq!(Instruction::In, Instruction::from_str("INBOX").unwrap());
        assert_eq!(Instruction::Out, Instruction::from_str("OUTBOX").unwrap());
        assert_eq!(Instruction::CopyFrom(VBMA::Pointer(1)), Instruction::from_str("COPYFROM 1").unwrap());
        assert_eq!(Instruction::CopyTo(VBMA::Pointer(12)), Instruction::from_str("COPYTO 12").unwrap());
        assert_eq!(Instruction::Add(VBMA::Pointer(0)), Instruction::from_str("ADD 0").unwrap());
        assert_eq!(Instruction::Sub(VBMA::Pointer(88)), Instruction::from_str("SUB 88").unwrap());
        assert_eq!(Instruction::BumpUp(VBMA::Pointer(5)), Instruction::from_str("BUMPUP 5").unwrap());
        assert_eq!(Instruction::BumpDown(VBMA::Pointer(9)), Instruction::from_str("BUMPDN 9").unwrap());
        assert_eq!(Instruction::Jump("0".to_string()), Instruction::from_str("JUMP 0").unwrap());
        assert_eq!(Instruction::JumpIfZero("b".to_string()), Instruction::from_str("JUMPZ b").unwrap());
        assert_eq!(Instruction::JumpIfNegative("cd".to_string()), Instruction::from_str("JUMPN cd").unwrap());
    }

    #[test]
    fn test_instructions_with_spaces() {
        assert_eq!(Instruction::In, Instruction::from_str(" INBOX ").unwrap());
        assert_eq!(Instruction::Out, Instruction::from_str("   OUTBOX ").unwrap());
        assert_eq!(Instruction::CopyFrom(VBMA::from_str("0").unwrap()), Instruction::from_str(" COPYFROM    0 ").unwrap());
        assert_eq!(Instruction::CopyTo(VBMA::from_str("0").unwrap()), Instruction::from_str("COPYTO   0 ").unwrap());
        assert_eq!(Instruction::Add(VBMA::from_str("0").unwrap()), Instruction::from_str("  ADD 0 ").unwrap());
        assert_eq!(Instruction::Sub(VBMA::from_str("0").unwrap()), Instruction::from_str("SUB   0 ").unwrap());
        assert_eq!(Instruction::BumpUp(VBMA::from_str("0").unwrap()), Instruction::from_str("  BUMPUP  0 ").unwrap());
        assert_eq!(Instruction::BumpDown(VBMA::from_str("0").unwrap()), Instruction::from_str("BUMPDN\n 0 ").unwrap());
        assert_eq!(Instruction::Jump("0".to_string()), Instruction::from_str("JUMP  0 ").unwrap());
        assert_eq!(Instruction::JumpIfZero("0".to_string()), Instruction::from_str(" JUMPZ  0 ").unwrap());
        assert_eq!(Instruction::JumpIfNegative("0".to_string()), Instruction::from_str("  JUMPN 0 ").unwrap());
    }

    #[test]
    fn test_pointer_instructions() {
        assert_eq!(Instruction::CopyFrom(VBMA::PointerAddress(10)), Instruction::from_str("COPYFROM [10]").unwrap());
        assert_eq!(Instruction::CopyTo(VBMA::PointerAddress(80)), Instruction::from_str("COPYTO [80]").unwrap());
        assert_eq!(Instruction::Add(VBMA::PointerAddress(0)), Instruction::from_str("ADD [0]").unwrap());
        assert_eq!(Instruction::Sub(VBMA::PointerAddress(88)), Instruction::from_str("SUB [88]").unwrap());
        assert_eq!(Instruction::BumpUp(VBMA::PointerAddress(5)), Instruction::from_str("BUMPUP [5]").unwrap());
        assert_eq!(Instruction::BumpDown(VBMA::PointerAddress(9)), Instruction::from_str("BUMPDN [9]").unwrap());
    }
}