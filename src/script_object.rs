use std::{collections::HashMap, str::FromStr};

pub mod instruction;
pub mod value_box;

use instruction::Instruction;

#[derive(Debug)]
/// The ScriptObject is the representation of the script.
/// It doesn't execute itself, nor it holds the state of the program.
/// It's a transcription of the text file that can be read by the interpreter.
pub struct ScriptObject {
    blocks: Vec<Block>,
    blocks_map: HashMap<String, usize>,
}

#[derive(Debug, PartialEq)]
/// A block is a set of instructions after a "jump point".
/// In a program without jumps, there is only one unnamed block.
pub struct Block {
    name: String,
    index: usize,
    pub instructions: Vec<Instruction>,
}

impl PartialEq for ScriptObject {
    fn eq(&self, other: &Self) -> bool {
        self.blocks == other.blocks
    }
}

impl ScriptObject {
    fn new(blocks: Vec<Block>) -> Self {
        let mut blocks_map = HashMap::new();
        for (i, block) in blocks.iter().enumerate() {
            blocks_map.insert(block.name.clone(), i);
        }

        Self { blocks, blocks_map }
    }

    /// Get the block at the given index.
    pub fn get_block_by_index(&self, current_block: usize) -> Option<&Block> {
        self.blocks.get(current_block)
    }

    /// Get the block with the given label.
    pub fn get_block_by_label(&self, label: &str) -> Option<&Block> {
        match self.blocks_map.get(label) {
            Some(index) => Some(&self.blocks[*index]),
            None => None,
        }
    }

    /// Get the next block after the given one (in the order of the script)
    pub fn get_next(&self, current_block: &Block) -> Option<&Block> {
        let curr_index = current_block.index;
        self.get_block_by_index(curr_index + 1)
    }
}

#[derive(Debug, thiserror::Error)]
/// After parsing the script, we can validate it.
/// This error is returned if the script is invalid.
pub enum ScriptObjectValidationError {
    #[error("Some jumps have invalid anchors")]
    InvalidJumps,
}

impl ScriptObject {
    /// After parsing the script, we can validate it.
    pub fn validate(&self) -> Result<(), ScriptObjectValidationError> {
        if !self.all_jumps_have_valid_anchors() {
            Err(ScriptObjectValidationError::InvalidJumps)
        } else {
            Ok(())
        }
    }

    /// Check if all jumps points to existing blocks.
    fn all_jumps_have_valid_anchors(&self) -> bool {
        let instructions = self
            .blocks
            .iter()
            .flat_map(|block| block.instructions.iter());
        for instruction in instructions {
            match instruction {
                Instruction::Jump(label)
                | Instruction::JumpIfZero(label)
                | Instruction::JumpIfNegative(label) => match self.get_block_by_label(label) {
                    Some(other) => {
                        if other.name != *label {
                            return false;
                        }
                    }
                    None => return false,
                },
                _ => {}
            }
        }

        true
    }
}

#[derive(Debug, thiserror::Error)]
/// Error that can occur when parsing the script.
pub enum ParseScriptObjectError {
    #[error(
        "PARSER ERROR | error parsing the script on line {line}: '{instruction}' | Detailed error: {error}"
    )]
    InvalidInstruction {
        line: usize,
        instruction: String,
        #[source]
        error: instruction::ParseInstructionError,
    },
}

impl FromStr for ScriptObject {
    type Err = ParseScriptObjectError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut blocks: Vec<Block> = Vec::new();
        blocks.push(Block {
            name: "entry".to_string(),
            index: 0,
            instructions: Vec::new(),
        });

        for (i, line) in s.lines().enumerate() {
            let line = line.trim();
            if line.starts_with("--") // Title
            || line.is_empty() // Empty line
            || line.contains("COMMENT")
            // Comment
            {
                continue;
            }

            if line.starts_with("DEFINE") {
                // Enter comment/label definition zone
                break;
            }

            let line_split_colon = line.split(':').collect::<Vec<&str>>();
            if line_split_colon.len() > 1 {
                // <=> line contains a colon
                // Block definition
                let new_block = Block {
                    name: line_split_colon[0].to_string(),
                    index: blocks.len(),
                    instructions: Vec::new(),
                };
                blocks.push(new_block);
                continue;
            }

            // Line is an instruction
            blocks
                .last_mut()
                .unwrap()
                .instructions
                .push(Instruction::from_str(line).map_err(|err| {
                    Self::Err::InvalidInstruction {
                        line: i + 1,
                        instruction: line.to_string(),
                        error: err,
                    }
                })?);
        }

        Ok(Self::new(blocks))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_script() {
        let script = "-- HUMAN RESOURCE MACHINE PROGRAM --

        a:
            INBOX   
            COPYTO   0
            INBOX   
            ADD      0
            OUTBOX  
            JUMP     a
        
        
        ";
        let script_object = ScriptObject::from_str(script).unwrap();

        let theorical_so = ScriptObject::new(vec![
            Block {
                name: "entry".to_string(),
                index: 0,
                instructions: vec![],
            },
            Block {
                name: "a".to_string(),
                index: 1,
                instructions: vec![
                    Instruction::In,
                    Instruction::CopyTo(value_box::ValueBoxMemoryAddress::Pointer(0)),
                    Instruction::In,
                    Instruction::Add(value_box::ValueBoxMemoryAddress::Pointer(0)),
                    Instruction::Out,
                    Instruction::Jump("a".to_string()),
                ],
            },
        ]);
        assert_eq!(script_object, theorical_so);
    }

    #[test]
    fn test_script_valid_anchors() {
        let script = "-- HUMAN RESOURCE MACHINE PROGRAM --

        a:
            JUMP     a
        b:
            JUMPZ    b
            JUMP     a
        c:
            JUMPN    b
        
        ";
        let script_object = ScriptObject::from_str(script).unwrap();

        assert!(script_object.all_jumps_have_valid_anchors());
    }

    #[test]
    fn test_script_invalid_anchors() {
        let script = "-- HUMAN RESOURCE MACHINE PROGRAM --

        a:
            JUMP     b
        b:
            JUMPZ    z
            JUMP     a
        c:
            JUMPN    b
        
        ";
        let script_object = ScriptObject::from_str(script).unwrap();

        assert!(!script_object.all_jumps_have_valid_anchors());
    }

    #[test]
    fn test_script_empty_block() {
        let script = "-- HUMAN RESOURCE MACHINE PROGRAM --

        a:
        b:
            JUMPZ    b
            JUMP     a
        c:
            JUMPN    b
        
        ";
        let script_object = ScriptObject::from_str(script).unwrap();

        assert!(script_object.get_block_by_label("a").is_some());
        assert!(script_object
            .get_block_by_label("a")
            .unwrap()
            .instructions
            .is_empty());
    }

    #[test]
    fn test_script_get_next() {
        let script = "-- HUMAN RESOURCE MACHINE PROGRAM --

        a:
        b:
            COPYTO   0
            JUMP     a
        c:
            JUMPN    b
        
        ";
        let script_object = ScriptObject::from_str(script).unwrap();

        assert_eq!(
            script_object.get_next(script_object.get_block_by_label("a").unwrap()),
            Some(script_object.get_block_by_label("b").unwrap())
        );
        assert_eq!(
            script_object.get_next(script_object.get_block_by_label("b").unwrap()),
            Some(script_object.get_block_by_label("c").unwrap())
        );
        assert_eq!(
            script_object.get_next(script_object.get_block_by_label("c").unwrap()),
            None
        );
    }
}
