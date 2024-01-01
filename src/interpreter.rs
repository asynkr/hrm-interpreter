use crate::script_object::{instruction::Instruction, value_box::ValueBox, Block, ScriptObject};

pub struct Interpreter {
    memory: Vec<Option<ValueBox>>,
    max_memory_size: usize,
    head: Option<ValueBox>,
    next_input: usize,
}

enum InstructionResult {
    JumpBlock(String),
    NextInstruction,
    Terminate,
}

enum BlockResult {
    JumpBlock(String),
    NextBlock,
    Terminate,
}

// Initializzation methods
impl Default for Interpreter {
    fn default() -> Self {
        Self {
            memory: vec![],
            max_memory_size: usize::MAX,
            head: None,
            next_input: 0,
        }
    }
}

impl Interpreter {
    pub fn with_memory(memory: Vec<Option<ValueBox>>, max_memory_size: usize) -> Self {
        Self {
            memory,
            max_memory_size,
            head: None,
            next_input: 0,
        }
    }

    pub fn with_memory_size(memory_size: usize) -> Self {
        Self {
            memory: vec![None; memory_size],
            max_memory_size: memory_size,
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
                let address = vbma.get_address(&self.memory).unwrap();
                if let Some(value) = self.memory.get(address) {
                    self.head = *value;
                    InstructionResult::NextInstruction
                } else {
                    panic!("No value in memory at address {}", address);
                }
            }
            Instruction::CopyTo(vbma) => {
                if self.head.is_none() {
                    panic!("No value in head");
                }

                let address = vbma.get_address(&self.memory).unwrap();
                if let Some(value) = self.memory.get_mut(address) {
                    *value = self.head;
                    InstructionResult::NextInstruction
                } else if address < self.max_memory_size {
                    // expand memory
                    self.memory.resize(address + 1, None);
                    // set value
                    self.memory[address] = self.head;
                    InstructionResult::NextInstruction
                } else {
                    panic!("{} is out of the memory bounds", address);
                }
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
