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

/// All the possible things that can happen after executing a block
enum BlockResult {
    /// A jump instruction was executed inside the block
    JumpBlock(String),
    /// The block reached its end, go to the next one
    NextBlock,
    /// The program has terminated.
    Terminate,
}

// Initialization methods
impl Interpreter {
    pub fn new(memory: Memory) -> Self {
        Self {
            memory,
            head: None,
            next_input: 0,
        }
    }
}

// Execution methods
impl Interpreter {
    pub fn execute(&mut self, script: &ScriptObject, inputs: &[ValueBox]) -> Vec<ValueBox> {
        let mut output: Vec<ValueBox> = vec![];
        let mut current_block: &Block = script.get_block_by_index(0).unwrap();

        loop {
            match self.execute_block(current_block, inputs, &mut output) {
                BlockResult::JumpBlock(label) => match script.get_block_by_label(&label) {
                    Some(block) => current_block = block,
                    None => panic!("Cannot jump: no block with label {} found", label),
                },
                BlockResult::NextBlock => match script.get_next(current_block) {
                    Some(block) => current_block = block,
                    None => break,
                },
                BlockResult::Terminate => break,
            }
        }

        output
    }

    fn execute_block(
        &mut self,
        block: &Block,
        inputs: &[ValueBox],
        outputs: &mut Vec<ValueBox>,
    ) -> BlockResult {
        for instruction in block.instructions.iter() {
            match self.execute_instruction(instruction, inputs, outputs) {
                InstructionResult::JumpBlock(label) => return BlockResult::JumpBlock(label),
                InstructionResult::NextInstruction => {}
                InstructionResult::Terminate => return BlockResult::Terminate,
            }
        }

        // All instructions executed
        // Go to next chronological block
        BlockResult::NextBlock
    }

    fn execute_instruction(
        &mut self,
        instruction: &Instruction,
        inputs: &[ValueBox],
        outputs: &mut Vec<ValueBox>,
    ) -> InstructionResult {
        match instruction {
            Instruction::In => {
                match inputs.get(self.next_input) {
                    Some(value) => {
                        self.next_input += 1;
                        self.head = Some(*value);
                    }
                    // No more inputs => terminate program
                    None => {
                        return InstructionResult::Terminate;
                    }
                }
            }
            Instruction::Out => match &self.head {
                Some(value) => {
                    outputs.push(*value);
                }
                None => panic!("Cannot output: None in head"),
            },
            Instruction::CopyFrom(vbma) => {
                let address = self.memory.get_valid_address(vbma).unwrap();
                if let Some(value) = self.memory.get(&address) {
                    self.head = Some(*value);
                } else {
                    panic!("No value in memory at address {}", address);
                }
            }
            Instruction::CopyTo(vbma) => {
                if self.head.is_none() {
                    panic!("Cannot copy to memory: None in head");
                }

                let address = self.memory.get_valid_address(vbma).unwrap();
                self.memory.set(&address, self.head);
            }

            Instruction::Add(vbma) => {
                let (vb_head, vm_mem) = self.get_head_and_mem_value(vbma);

                match (vb_head, vm_mem) {
                    (ValueBox::Number(h), ValueBox::Number(m)) => self.head = Some(ValueBox::from(h + m)),
                    (ValueBox::Character(_), ValueBox::Character(_)) => panic!("Cannot add characters (head: {:?} and mem: {:?} at adress {:?})", vb_head, vm_mem, self.memory.get_valid_address(vbma)),
                    _ => panic!("Cannot add characters and numbers together (head: {:?} and mem: {:?} at adress {:?})", vb_head, vm_mem, self.memory.get_valid_address(vbma)),
                }
            }
            Instruction::Sub(vbma) => {
                let (vb_head, vm_mem) = self.get_head_and_mem_value(vbma);

                match (vb_head, vm_mem) {
                    (ValueBox::Number(h), ValueBox::Number(m)) => self.head = Some(ValueBox::from(h - m)),
                    (ValueBox::Character(h), ValueBox::Character(m)) => {
                        // Special case: in HRM, we CAN subtract characters together
                        // The result is the distance between the two characters in the alphabet (an integer)
                        let get_alphabetic_index = |c: char| -> i8 {
                            let c = c.to_ascii_uppercase();
                            c as i8 - 'A' as i8
                        };
                        let h = get_alphabetic_index(h);
                        let m = get_alphabetic_index(m);
                        let result = (h - m) as i32;
                        self.head = Some(ValueBox::from(result));
                    }
                    _ => panic!("Cannot subtract characters and numbers together (head: {:?} and mem: {:?} at adress {:?})", vb_head, vm_mem, self.memory.get_valid_address(vbma)),
                }
            }

            Instruction::BumpUp(vbma) => self.bump_mem_value(vbma, true),
            Instruction::BumpDown(vbma) => self.bump_mem_value(vbma, false),

            Instruction::Jump(block_key) => return InstructionResult::JumpBlock(block_key.clone()),
            Instruction::JumpIfZero(block_key) => match self.head {
                Some(ValueBox::Number(n)) => {
                    if n == 0 {
                        return InstructionResult::JumpBlock(block_key.clone());
                    }
                }
                _ => panic!(
                    "Cannot test IfZero if head ({:?}) is not a valid number",
                    self.head
                ),
            },
            Instruction::JumpIfNegative(block_key) => match self.head {
                Some(ValueBox::Number(n)) => {
                    if n < 0 {
                        return InstructionResult::JumpBlock(block_key.clone());
                    }
                }
                _ => panic!(
                    "Cannot test IfNegative if head ({:?}) is not a valid number",
                    self.head
                ),
            },
        };
        InstructionResult::NextInstruction
    }

    fn get_head_and_mem_value(&mut self, vbma: &ValueBoxMemoryAddress) -> (ValueBox, ValueBox) {
        let address = self.memory.get_valid_address(vbma).unwrap();
        let mem_value = self.memory.get(&address);

        match (self.head, mem_value) {
            (Some(vb_head), Some(vb_mem)) => (vb_head, *vb_mem),
            _ => panic!(
                "Cannot retrieve head or memory (head: {:?} and mem: {:?} at adress {:?})",
                self.head,
                mem_value,
                self.memory.get_valid_address(vbma)
            ),
        }
    }

    fn bump_mem_value(&mut self, vbma: &ValueBoxMemoryAddress, up: bool) {
        let address = self.memory.get_valid_address(vbma).unwrap();
        let mem_value = self.memory.get(&address);

        let new_value = match (mem_value, up) {
            (Some(ValueBox::Number(m)), true) => *m + 1,
            (Some(ValueBox::Number(m)), false) => *m - 1,
            _ => panic!(
                "Cannot bump up, invalid value in memory: {:?} at address: {:?} (vbma: {:?})",
                mem_value, address, vbma
            ),
        };

        self.memory.set(&address, Some(ValueBox::from(new_value)));
        self.head = Some(ValueBox::from(new_value));
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
        assert_eq!(result, InstructionResult::Terminate);

        let result =
            interpreter.execute_instruction(&Instruction::In, &[ValueBox::from(10)], &mut vec![]);
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::JumpBlock("label".to_string()));
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
        assert_eq!(result, InstructionResult::JumpBlock("label".to_string()));
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
        assert_eq!(result, InstructionResult::NextInstruction);
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
        assert_eq!(result, InstructionResult::JumpBlock("label".to_string()));
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
        assert_eq!(result, InstructionResult::NextInstruction);
    }
}
