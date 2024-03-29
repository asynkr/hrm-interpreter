use std::str::FromStr;

use collapse::collapse;

use super::value_box::{self, ParseValueBoxMemoryAddressError};

use value_box::ValueBoxMemoryAddress as ValBoxMemAddr;
type BlockKey = String;

#[derive(Debug, PartialEq)]
/// An instruction is a line of code in the script.
/// It holds the operation and sometimes some additional data.
/// The rust enum structure is perfect for this.
///
/// In this implementation, the instruction doesn't describe how it should be executed,
/// that's the job of the interpreter.
pub enum Instruction {
    /// Read the next input ValueBox from the input belt
    In,
    /// Drop the head on the output belt
    Out,

    /// Copy the value at the given memory address to the head
    CopyFrom(ValBoxMemAddr),
    /// Copy the head to the given memory address
    CopyTo(ValBoxMemAddr),

    /// Add the value at the given memory address to the head
    Add(ValBoxMemAddr),
    /// Subtract the value at the given memory address from the head (ie head - value)
    Sub(ValBoxMemAddr),
    /// Add 1 to the value at the given memory address. The result is written at the same address AND in the head.
    BumpUp(ValBoxMemAddr),
    /// Subtract 1 to the value at the given memory address. The result is written at the same address AND in the head.
    BumpDown(ValBoxMemAddr),

    /// Jump to the given bock
    Jump(BlockKey),
    /// Jump to the given block if the head is zero
    JumpIfZero(BlockKey),
    /// Jump to the given block if the head is (strictly) negative
    JumpIfNegative(BlockKey),
}

#[derive(Debug, thiserror::Error)]
/// Error that can occur when parsing an instruction.
pub enum ParseInstructionError {
    #[error("too much parts in the instruction line, expected 2 at most, got {}", .0.len())]
    TooMuchParts(Vec<String>),
    #[error("{0} is not a valid instruction")]
    InvalidInstruction(String),
    #[error("instruction has an invalid memory address:\n\t{0}")]
    InvalidMemoryAddress(#[from] ParseValueBoxMemoryAddressError),
}

impl FromStr for Instruction {
    type Err = ParseInstructionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: &str = &collapse(s);
        let parts = s.split_whitespace().collect::<Vec<&str>>();

        if parts.len() > 2 {
            return Err(Self::Err::TooMuchParts(
                parts.iter().map(|s| s.to_string()).collect(),
            ));
        }

        #[allow(clippy::get_first)]
        let instruction_key = *parts.get(0).unwrap();
        let address_key = parts.get(1).cloned();

        match (instruction_key, address_key) {
            ("INBOX", None) => Ok(Instruction::In),
            ("OUTBOX", None) => Ok(Instruction::Out),
            ("COPYFROM", Some(akey)) => Ok(Instruction::CopyFrom(ValBoxMemAddr::from_str(akey)?)),
            ("COPYTO", Some(akey)) => Ok(Instruction::CopyTo(ValBoxMemAddr::from_str(akey)?)),
            ("ADD", Some(akey)) => Ok(Instruction::Add(ValBoxMemAddr::from_str(akey)?)),
            ("SUB", Some(akey)) => Ok(Instruction::Sub(ValBoxMemAddr::from_str(akey)?)),
            ("BUMPUP", Some(akey)) => Ok(Instruction::BumpUp(ValBoxMemAddr::from_str(akey)?)),
            ("BUMPDN", Some(akey)) => Ok(Instruction::BumpDown(ValBoxMemAddr::from_str(akey)?)),
            ("JUMP", Some(akey)) => Ok(Instruction::Jump(akey.to_string())),
            ("JUMPZ", Some(akey)) => Ok(Instruction::JumpIfZero(akey.to_string())),
            ("JUMPN", Some(akey)) => Ok(Instruction::JumpIfNegative(akey.to_string())),
            _ => Err(Self::Err::InvalidInstruction(s.to_string())),
        }
    }
}

#[cfg(test)]
mod instruction_tests {
    use super::*;

    #[test]
    fn test_instruction_from_str() {
        assert_eq!(Instruction::In, Instruction::from_str("INBOX").unwrap());
        assert_eq!(Instruction::Out, Instruction::from_str("OUTBOX").unwrap());
        assert_eq!(
            Instruction::CopyFrom(ValBoxMemAddr::Pointer(1)),
            Instruction::from_str("COPYFROM 1").unwrap()
        );
        assert_eq!(
            Instruction::CopyTo(ValBoxMemAddr::Pointer(12)),
            Instruction::from_str("COPYTO 12").unwrap()
        );
        assert_eq!(
            Instruction::Add(ValBoxMemAddr::Pointer(0)),
            Instruction::from_str("ADD 0").unwrap()
        );
        assert_eq!(
            Instruction::Sub(ValBoxMemAddr::Pointer(88)),
            Instruction::from_str("SUB 88").unwrap()
        );
        assert_eq!(
            Instruction::BumpUp(ValBoxMemAddr::Pointer(5)),
            Instruction::from_str("BUMPUP 5").unwrap()
        );
        assert_eq!(
            Instruction::BumpDown(ValBoxMemAddr::Pointer(9)),
            Instruction::from_str("BUMPDN 9").unwrap()
        );
        assert_eq!(
            Instruction::Jump("0".to_string()),
            Instruction::from_str("JUMP 0").unwrap()
        );
        assert_eq!(
            Instruction::JumpIfZero("b".to_string()),
            Instruction::from_str("JUMPZ b").unwrap()
        );
        assert_eq!(
            Instruction::JumpIfNegative("cd".to_string()),
            Instruction::from_str("JUMPN cd").unwrap()
        );
    }

    #[test]
    fn test_instructions_with_spaces() {
        assert_eq!(Instruction::In, Instruction::from_str(" INBOX ").unwrap());
        assert_eq!(
            Instruction::Out,
            Instruction::from_str("   OUTBOX ").unwrap()
        );
        assert_eq!(
            Instruction::CopyFrom(ValBoxMemAddr::from_str("0").unwrap()),
            Instruction::from_str(" COPYFROM    0 ").unwrap()
        );
        assert_eq!(
            Instruction::CopyTo(ValBoxMemAddr::from_str("0").unwrap()),
            Instruction::from_str("COPYTO   0 ").unwrap()
        );
        assert_eq!(
            Instruction::Add(ValBoxMemAddr::from_str("0").unwrap()),
            Instruction::from_str("  ADD 0 ").unwrap()
        );
        assert_eq!(
            Instruction::Sub(ValBoxMemAddr::from_str("0").unwrap()),
            Instruction::from_str("SUB   0 ").unwrap()
        );
        assert_eq!(
            Instruction::BumpUp(ValBoxMemAddr::from_str("0").unwrap()),
            Instruction::from_str("  BUMPUP  0 ").unwrap()
        );
        assert_eq!(
            Instruction::BumpDown(ValBoxMemAddr::from_str("0").unwrap()),
            Instruction::from_str("BUMPDN\n 0 ").unwrap()
        );
        assert_eq!(
            Instruction::Jump("0".to_string()),
            Instruction::from_str("JUMP  0 ").unwrap()
        );
        assert_eq!(
            Instruction::JumpIfZero("0".to_string()),
            Instruction::from_str(" JUMPZ  0 ").unwrap()
        );
        assert_eq!(
            Instruction::JumpIfNegative("0".to_string()),
            Instruction::from_str("  JUMPN 0 ").unwrap()
        );
    }

    #[test]
    fn test_pointer_instructions() {
        assert_eq!(
            Instruction::CopyFrom(ValBoxMemAddr::PointerAddress(10)),
            Instruction::from_str("COPYFROM [10]").unwrap()
        );
        assert_eq!(
            Instruction::CopyTo(ValBoxMemAddr::PointerAddress(80)),
            Instruction::from_str("COPYTO [80]").unwrap()
        );
        assert_eq!(
            Instruction::Add(ValBoxMemAddr::PointerAddress(0)),
            Instruction::from_str("ADD [0]").unwrap()
        );
        assert_eq!(
            Instruction::Sub(ValBoxMemAddr::PointerAddress(88)),
            Instruction::from_str("SUB [88]").unwrap()
        );
        assert_eq!(
            Instruction::BumpUp(ValBoxMemAddr::PointerAddress(5)),
            Instruction::from_str("BUMPUP [5]").unwrap()
        );
        assert_eq!(
            Instruction::BumpDown(ValBoxMemAddr::PointerAddress(9)),
            Instruction::from_str("BUMPDN [9]").unwrap()
        );
    }
}
