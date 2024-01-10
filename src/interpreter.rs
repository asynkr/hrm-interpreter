use std::fmt::Debug;

use crate::script_object::{
    instruction::Instruction,
    value_box::{ValueBox, ValueBoxMemoryAddress},
    Block, ScriptObject,
};

pub mod memory;

use self::memory::Memory;

/// The interpreter is the component that executes the script.
/// It holds the state of the program.
pub struct Interpreter {
    /// The tiles on the floor where ValueBoxes can be placed
    memory: Memory,
    /// The eventual ValueBox held by the character
    head: Option<ValueBox>,
    /// The index of the next input ValueBox to be read
    next_input: usize,
}

/// Holds the state of the interpreter at a given moment,
/// for debugging purposes.
pub struct InterpreterStateInfo {
    inputs_left: Vec<String>,
    outputs: Vec<String>,
    memory: Vec<(usize, String)>,
}

impl Debug for InterpreterStateInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inputs_left = self
            .inputs_left
            .iter()
            .map(|vb| vb.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let outputs = self
            .outputs
            .iter()
            .map(|vb| vb.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let memory = self
            .memory
            .iter()
            .map(|(address, vb)| format!("{}: {}", address, vb))
            .collect::<Vec<String>>()
            .join("\n");

        write!(
            f,
            "Inputs left: {}\nOutputs: {}\nMemory:\n{}",
            inputs_left, outputs, memory
        )
    }
}

impl Interpreter {
    pub fn new(memory: Memory) -> Self {
        Self {
            memory,
            head: None,
            next_input: 0,
        }
    }

    fn build_state(&self, inputs: &[ValueBox], outputs: &[ValueBox]) -> InterpreterStateInfo {
        let inputs_left = inputs[self.next_input..]
            .iter()
            .map(|vb| vb.to_string())
            .collect::<Vec<String>>();
        let outputs = outputs
            .iter()
            .map(|vb| vb.to_string())
            .collect::<Vec<String>>();

        let memory_indices = 0..self.memory.get_max_address() + 1;
        let memory = memory_indices
            .map(|i| {
                if let Some(vb) = self.memory.get(&i) {
                    (i, vb.to_string())
                } else {
                    (i, "None".to_string())
                }
            })
            .collect::<Vec<(usize, String)>>();

        InterpreterStateInfo {
            inputs_left,
            outputs,
            memory,
        }
    }
}

// ==================== Script execution ====================

#[derive(Debug, thiserror::Error)]
/// Wrapper for all the possible errors that can occur when executing a script.
pub enum ExecuteScriptError {
    #[error("INTERPRETER ERROR | cannot jump: no block with label {1} found\n-- STATE --\n{0:?}")]
    InvalidJumpError(InterpreterStateInfo, String),
    #[error("INTERPRETER ERROR | error executing an instruction:\n\t{1}\n-- STATE --\n{0:?}")]
    ExecuteInstructionError(InterpreterStateInfo, #[source] ExecuteInstructionError),
}

impl Interpreter {
    /// Execute a given script with given outputs, starting at first block.
    pub fn execute(
        &mut self,
        script: &ScriptObject,
        inputs: &[ValueBox],
    ) -> Result<Vec<ValueBox>, ExecuteScriptError> {
        let mut output: Vec<ValueBox> = vec![];
        let mut current_block: &Block = script.get_block_by_index(0).unwrap();

        loop {
            match self
                .execute_block(current_block, inputs, &mut output)
                .map_err(|e| {
                    ExecuteScriptError::ExecuteInstructionError(
                        self.build_state(inputs, &output),
                        e,
                    )
                })? {
                BlockResult::JumpBlock(label) => match script.get_block_by_label(&label) {
                    Some(block) => current_block = block,
                    None => {
                        return Err(ExecuteScriptError::InvalidJumpError(
                            self.build_state(inputs, &output),
                            label,
                        ))
                    }
                },
                BlockResult::NextBlock => match script.get_next(current_block) {
                    Some(block) => current_block = block,
                    None => break,
                },
                BlockResult::Terminate => break,
            }
        }

        Ok(output)
    }
}

// ==================== Block execution ====================

/// All the possible things that can happen after executing a block
enum BlockResult {
    /// A jump instruction was executed inside the block
    JumpBlock(String),
    /// The block reached its end, go to the next one
    NextBlock,
    /// The program has terminated.
    Terminate,
}

impl Interpreter {
    /// Execute the instructions of a given block one by one,
    /// mutating the output along the way.
    fn execute_block(
        &mut self,
        block: &Block,
        inputs: &[ValueBox],
        outputs: &mut Vec<ValueBox>,
    ) -> Result<BlockResult, ExecuteInstructionError> {
        for instruction in block.instructions.iter() {
            match self.execute_instruction(instruction, inputs, outputs)? {
                InstructionResult::JumpBlock(label) => return Ok(BlockResult::JumpBlock(label)),
                InstructionResult::NextInstruction => {}
                InstructionResult::Terminate => return Ok(BlockResult::Terminate),
            }
        }

        // All instructions executed
        // Go to next chronological block
        Ok(BlockResult::NextBlock)
    }
}

// ==================== Instruction execution ====================

#[derive(Debug, PartialEq)]
/// All the possible things that can happen after executing an instruction
enum InstructionResult {
    /// A jump instruction was executed
    JumpBlock(String),
    /// The instruction was successfully executed, read the next one
    NextInstruction,
    /// The program has terminated.
    /// (Can happen if an INBOX instruction is executed with no more inputs to read)
    Terminate,
}

#[derive(Debug, thiserror::Error)]
/// All errors that can occur when executing an instruction
/// Errors are voluntarily redundant from one instruction type to another,
/// to make it easier to understand what went wrong.
pub enum ExecuteInstructionError {
    #[error("cannot output: head empty")]
    OutputNone,

    #[error("cannot copy from:\n\t{0}")]
    CopyFromInvalidAddress(#[source] memory::GetMemoryError),
    #[error("cannot copy to:\n\t{0}")]
    CopyToInvalidAddress(#[source] memory::SetMemoryError),
    #[error("cannot copy to: head empty")]
    CopyToHeadNone,

    #[error("cannot read memory value from VBMA:\n\t{0}")]
    AddInvalidAddress(#[source] memory::GetMemoryError),
    #[error("cannot add: empty head")]
    AddHeadNone,
    #[error("cannot add characters (head: {head} and mem: {mem} at address {address})")]
    AddCharacters {
        head: char,
        mem: char,
        address: usize,
    },
    #[error("cannot add characters and numbers together (head: {head:?} and mem: {mem:?} at address {address})")]
    AddCharacterAndNumber {
        head: ValueBox,
        mem: ValueBox,
        address: usize,
    },

    #[error("cannot read memory value from VBMA:\n\t{0}")]
    SubInvalidAddress(#[source] memory::GetMemoryError),
    #[error("cannot subtract: empty head")]
    SubHeadNone,
    #[error("cannot subtract characters and numbers together (head: {head:?} and mem: {mem:?} at address {address})")]
    SubCharacterAndNumber {
        head: ValueBox,
        mem: ValueBox,
        address: usize,
    },

    #[error("cannot test IfZero if head ({0:?}) is not a valid number")]
    JumpIfZeroInvalidHead(Option<ValueBox>),
    #[error("cannot test IfNegative if head ({0:?}) is not a valid number")]
    JumpIfNegativeInvalidHead(Option<ValueBox>),

    #[error("cannot bump memory value from VBMA:\n\t{0}")]
    BumpInvalidAddress(#[source] memory::GetMemoryError),
    #[error("cannot bump a character")]
    BumpCharacter,
}

impl Interpreter {
    /// Execute 1 instruction
    /// using one big match to handle all the possible instructions
    fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        inputs: &[ValueBox],
        outputs: &mut Vec<ValueBox>,
    ) -> Result<InstructionResult, ExecuteInstructionError> {
        match instruction {
            Instruction::In => {
                match inputs.get(self.next_input) {
                    Some(value) => {
                        self.next_input += 1;
                        self.head = Some(*value);
                    }
                    // No more inputs => terminate program
                    None => {
                        return Ok(InstructionResult::Terminate);
                    }
                }
            }
            Instruction::Out => match &self.head {
                Some(value) => outputs.push(*value),
                None => return Err(ExecuteInstructionError::OutputNone),
            },
            Instruction::CopyFrom(vbma) => {
                let value = self
                    .memory
                    .get_with_vbma(vbma)
                    .map_err(ExecuteInstructionError::CopyFromInvalidAddress)?;
                self.head = Some(*value);
            }
            Instruction::CopyTo(_) if self.head.is_none() => {
                return Err(ExecuteInstructionError::CopyToHeadNone);
            }
            Instruction::CopyTo(vbma) => {
                let head_value = self.head.ok_or(ExecuteInstructionError::CopyToHeadNone)?;

                self.memory
                    .set_with_vbma(vbma, Some(head_value))
                    .map_err(ExecuteInstructionError::CopyToInvalidAddress)?;
            }

            Instruction::Add(vbma) => {
                let mem_value = self
                    .memory
                    .get_with_vbma(vbma)
                    .map_err(ExecuteInstructionError::AddInvalidAddress)?;
                let head_value = &self.head.ok_or(ExecuteInstructionError::AddHeadNone)?;

                match (head_value, mem_value) {
                    (ValueBox::Number(h), ValueBox::Number(m)) => {
                        self.head = Some(ValueBox::from(h + m))
                    }
                    (ValueBox::Character(char_head), ValueBox::Character(char_mem)) => {
                        return Err(ExecuteInstructionError::AddCharacters {
                            head: *char_head,
                            mem: *char_mem,
                            address: self.memory.translate_vbma_to_mem_address(vbma).unwrap(),
                        });
                    }
                    _ => {
                        return Err(ExecuteInstructionError::AddCharacterAndNumber {
                            head: *head_value,
                            mem: *mem_value,
                            address: self.memory.translate_vbma_to_mem_address(vbma).unwrap(),
                        });
                    }
                }
            }
            Instruction::Sub(vbma) => {
                let mem_value = self
                    .memory
                    .get_with_vbma(vbma)
                    .map_err(ExecuteInstructionError::SubInvalidAddress)?;
                let head_value = &self.head.ok_or(ExecuteInstructionError::SubHeadNone)?;

                match (head_value, mem_value) {
                    (ValueBox::Number(h), ValueBox::Number(m)) => {
                        self.head = Some(ValueBox::from(h - m))
                    }
                    (ValueBox::Character(h), ValueBox::Character(m)) => {
                        // Special case: in HRM, we CAN subtract characters together
                        // The result is the distance between the two characters in the alphabet (an integer)
                        let get_alphabetic_index = |c: &char| -> i8 {
                            let c = c.to_ascii_uppercase();
                            c as i8 - 'A' as i8
                        };
                        let h = get_alphabetic_index(h);
                        let m = get_alphabetic_index(m);
                        let result = (h - m) as i32;
                        self.head = Some(ValueBox::from(result));
                    }
                    _ => {
                        return Err(ExecuteInstructionError::SubCharacterAndNumber {
                            head: *head_value,
                            mem: *mem_value,
                            address: self.memory.translate_vbma_to_mem_address(vbma).unwrap(),
                        });
                    }
                }
            }

            Instruction::BumpUp(vbma) => self.bump_mem_value(vbma, true)?,
            Instruction::BumpDown(vbma) => self.bump_mem_value(vbma, false)?,

            Instruction::Jump(block_key) => {
                return Ok(InstructionResult::JumpBlock(block_key.clone()))
            }
            Instruction::JumpIfZero(block_key) => match self.head {
                Some(ValueBox::Number(0)) => {
                    return Ok(InstructionResult::JumpBlock(block_key.clone()));
                }
                Some(ValueBox::Character(_)) => {} // Characters are never equal to 0
                Some(ValueBox::Number(_)) => {}    // Number != 0 => do nothing
                _ => {
                    return Err(ExecuteInstructionError::JumpIfZeroInvalidHead(self.head));
                }
            },
            Instruction::JumpIfNegative(block_key) => match self.head {
                Some(ValueBox::Number(n)) if n < 0 => {
                    return Ok(InstructionResult::JumpBlock(block_key.clone()));
                }
                Some(ValueBox::Character(_)) => {} // Characters are never negative
                Some(ValueBox::Number(_)) => {}    // Number >= 0 => do nothing
                _ => {
                    return Err(ExecuteInstructionError::JumpIfNegativeInvalidHead(
                        self.head,
                    ));
                }
            },
        };
        Ok(InstructionResult::NextInstruction)
    }

    fn bump_mem_value(
        &mut self,
        vbma: &ValueBoxMemoryAddress,
        up: bool,
    ) -> Result<(), ExecuteInstructionError> {
        let mem_value = self
            .memory
            .get_with_vbma(vbma)
            .map_err(ExecuteInstructionError::BumpInvalidAddress)?;

        let new_value = match mem_value {
            ValueBox::Number(m) if up => m + 1,
            ValueBox::Number(m) => m - 1,
            ValueBox::Character(_) => return Err(ExecuteInstructionError::BumpCharacter),
        };

        self.memory
            .set_with_vbma(vbma, Some(ValueBox::from(new_value)))
            .unwrap(); // Should never fail because we just read it
        self.head = Some(ValueBox::from(new_value));
        Ok(())
    }
}

#[cfg(test)]
mod test_instructions_execution {
    use std::collections::HashMap;

    use super::*;
    use crate::script_object::value_box::ValueBoxMemoryAddress;

    #[test]
    fn test_inbox() {
        let mut interpreter = Interpreter {
            memory: Memory::default(),
            head: None,
            next_input: 0,
        };

        let result = interpreter.execute_instruction(&Instruction::In, &[], &mut vec![]);
        assert_eq!(result.unwrap(), InstructionResult::Terminate);

        let result =
            interpreter.execute_instruction(&Instruction::In, &[ValueBox::from(10)], &mut vec![]);
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
        assert_eq!(interpreter.head, Some(ValueBox::from(10)));
    }

    #[test]
    fn test_outbox() {
        let mut interpreter = Interpreter {
            memory: Memory::default(),
            head: Some(ValueBox::from(42)),
            next_input: 0,
        };

        let mut outputs = vec![];
        let result = interpreter.execute_instruction(&Instruction::Out, &[], &mut outputs);
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
        assert_eq!(outputs, vec![ValueBox::from(42)]);
    }

    #[test]
    fn test_copy_from() {
        let mut interpreter = Interpreter {
            memory: Memory::with_data(HashMap::from_iter([(0, ValueBox::from(42))]), 10),
            head: None,
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::CopyFrom(ValueBoxMemoryAddress::Pointer(0)),
            &[],
            &mut vec![],
        );
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
        assert_eq!(interpreter.head, Some(ValueBox::from(42)));
    }

    #[test]
    fn test_copy_to() {
        let mut interpreter = Interpreter {
            memory: Memory::with_data(HashMap::from_iter([(0, ValueBox::from(42))]), 10),
            head: Some(ValueBox::from(10)),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::CopyTo(ValueBoxMemoryAddress::Pointer(0)),
            &[],
            &mut vec![],
        );
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
        assert_eq!(interpreter.memory.get(&0), Some(&ValueBox::from(10)));
    }

    #[test]
    fn test_add() {
        let mut interpreter = Interpreter {
            memory: Memory::with_data(HashMap::from_iter([(0, ValueBox::from(42))]), 10),
            head: Some(ValueBox::from(10)),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::Add(ValueBoxMemoryAddress::Pointer(0)),
            &[],
            &mut vec![],
        );
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
        assert_eq!(interpreter.head, Some(ValueBox::from(52)));
        assert_eq!(interpreter.memory.get(&0), Some(&ValueBox::from(42)));
    }

    #[test]
    fn test_sub() {
        let mut interpreter = Interpreter {
            memory: Memory::with_data(HashMap::from_iter([(0, ValueBox::from(42))]), 10),
            head: Some(ValueBox::from(10)),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::Sub(ValueBoxMemoryAddress::Pointer(0)),
            &[],
            &mut vec![],
        );
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
        assert_eq!(interpreter.head, Some(ValueBox::from(-32)));
        assert_eq!(interpreter.memory.get(&0), Some(&ValueBox::from(42)));
    }

    #[test]
    fn test_sub_characters() {
        let mut interpreter = Interpreter {
            memory: Memory::with_data(HashMap::from_iter([(0, ValueBox::from('E'))]), 10),
            head: Some(ValueBox::from('A')),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::Sub(ValueBoxMemoryAddress::Pointer(0)),
            &[],
            &mut vec![],
        );
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
        assert_eq!(interpreter.head, Some(ValueBox::from(-4)));
        assert_eq!(interpreter.memory.get(&0), Some(&ValueBox::from('E')));
    }

    #[test]
    fn test_bump_up() {
        let mut interpreter = Interpreter {
            memory: Memory::with_data(HashMap::from_iter([(0, ValueBox::from(42))]), 10),
            head: Some(ValueBox::from(10)),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::BumpUp(ValueBoxMemoryAddress::Pointer(0)),
            &[],
            &mut vec![],
        );
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
        assert_eq!(interpreter.head, Some(ValueBox::from(43)));
        assert_eq!(interpreter.memory.get(&0), Some(&ValueBox::from(43)));
    }

    #[test]
    fn test_bump_down() {
        let mut interpreter = Interpreter {
            memory: Memory::with_data(HashMap::from_iter([(0, ValueBox::from(42))]), 10),
            head: Some(ValueBox::from(10)),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::BumpDown(ValueBoxMemoryAddress::Pointer(0)),
            &[],
            &mut vec![],
        );
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
        assert_eq!(interpreter.head, Some(ValueBox::from(41)));
        assert_eq!(interpreter.memory.get(&0), Some(&ValueBox::from(41)));
    }

    #[test]
    fn test_jump() {
        let mut interpreter = Interpreter {
            memory: Memory::default(),
            head: None,
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::Jump("label".to_string()),
            &[],
            &mut vec![],
        );
        assert_eq!(
            result.unwrap(),
            InstructionResult::JumpBlock("label".to_string())
        );
    }

    #[test]
    fn test_jump_if_zero() {
        let mut interpreter = Interpreter {
            memory: Memory::default(),
            head: Some(ValueBox::from(0)),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::JumpIfZero("label".to_string()),
            &[],
            &mut vec![],
        );
        assert_eq!(
            result.unwrap(),
            InstructionResult::JumpBlock("label".to_string())
        );
    }

    #[test]
    fn test_jump_if_zero_not_zero() {
        let mut interpreter = Interpreter {
            memory: Memory::default(),
            head: Some(ValueBox::from(42)),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::JumpIfZero("label".to_string()),
            &[],
            &mut vec![],
        );
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
    }

    #[test]
    fn test_jump_if_negative() {
        let mut interpreter = Interpreter {
            memory: Memory::default(),
            head: Some(ValueBox::from(-42)),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::JumpIfNegative("label".to_string()),
            &[],
            &mut vec![],
        );
        assert_eq!(
            result.unwrap(),
            InstructionResult::JumpBlock("label".to_string())
        );
    }

    #[test]
    fn test_jump_if_negative_not_negative() {
        let mut interpreter = Interpreter {
            memory: Memory::default(),
            head: Some(ValueBox::from(0)),
            next_input: 0,
        };

        let result = interpreter.execute_instruction(
            &Instruction::JumpIfNegative("label".to_string()),
            &[],
            &mut vec![],
        );
        assert_eq!(result.unwrap(), InstructionResult::NextInstruction);
    }
}
