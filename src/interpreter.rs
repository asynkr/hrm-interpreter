use crate::script_object::{instruction::Instruction, value_box::ValueBox, Block, ScriptObject};

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
                        InstructionResult::NextInstruction
                    }
                    // No more inputs => terminate program
                    None => InstructionResult::Terminate,
                }
            }
            Instruction::Out => match &self.head {
                Some(value) => {
                    outputs.push(*value);
                    InstructionResult::NextInstruction
                }
                None => panic!("No value in head"),
            },
            Instruction::CopyFrom(vbma) => {
                let address = self.memory.get_valid_address(vbma).unwrap();
                if let Some(value) = self.memory.get(&address) {
                    self.head = Some(*value);
                    InstructionResult::NextInstruction
                } else {
                    panic!("No value in memory at address {}", address);
                }
            }
            Instruction::CopyTo(vbma) => {
                if self.head.is_none() {
                    panic!("No value in head");
                }

                let address = self.memory.get_valid_address(vbma).unwrap();
                self.memory.set(&address, self.head);
                InstructionResult::NextInstruction
            }

            Instruction::Add(_) => todo!(),
            Instruction::Sub(_) => todo!(),
            Instruction::BumpUp(_) => todo!(),
            Instruction::BumpDown(_) => todo!(),
            Instruction::Jump(_) => todo!(),
            Instruction::JumpIfZero(_) => todo!(),
            Instruction::JumpIfNegative(_) => todo!(),
        }
    }
}
