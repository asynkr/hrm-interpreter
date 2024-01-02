use std::{collections::HashMap, error::Error};

use crate::script_object::value_box::{ValueBox, ValueBoxMemoryAddress};

/// The memory is the component that holds the ValueBoxes placed on the floor.
/// A key feature of Human Resource Machine is that the memory can be (very) limited in size.
pub struct Memory {
    data: HashMap<usize, ValueBox>,
    max_address: usize,
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
            max_address: usize::MAX,
        }
    }
}

// General methods
impl Memory {
    pub fn with_data(data: HashMap<usize, ValueBox>, max_address: usize) -> Self {
        #[cfg(debug_assertions)]
        {
            for address in data.keys() {
                if address > &max_address {
                    panic!("WARNING! You have built a memory with at least 1 invalid memory address:Memory address {address} out of bounds (accepted: [1, {}])", max_address);
                }
            }
        }
        Self { data, max_address }
    }

    pub fn get(&self, address: &usize) -> Option<&ValueBox> {
        self.data.get(address)
    }

    /// at_adress is at least 1 and at most max_len - 1
    pub fn can_set(&self, at_address: &usize) -> bool {
        at_address <= &self.max_address
    }

    /// check if at_address is a valid memory address when at_address is not usize
    pub fn could_set(&self, at_address: &i32) -> bool {
        at_address >= &0 && (*at_address as usize) <= self.max_address
    }

    pub fn set(&mut self, address: &usize, value: Option<ValueBox>) {
        if !self.can_set(address) {
            // address is bound by max_len
            panic!(
                "Memory address {address} out of bounds (accepted: [1, {}])",
                self.max_address
            );
        }

        match value {
            Some(value) => {
                self.data.insert(*address, value);
            }
            None => {
                self.data.remove(address);
            }
        }
    }
}

// Specific methods
impl Memory {
    pub fn get_valid_address(
        &self,
        value_box_memory_address: &ValueBoxMemoryAddress,
    ) -> Result<usize, Box<dyn Error>> {
        match value_box_memory_address {
            ValueBoxMemoryAddress::Pointer(address) => Ok(*address),
            ValueBoxMemoryAddress::PointerAddress(pointer_address) => {
                let address = self.get(pointer_address);
                if let Some(ValueBox::Number(n)) = address {
                    if !self.could_set(n) {
                        return Err(format!("Value in memory at {pointer_address} is not a valid memory address ({n})").into());
                    }
                    Ok(*n as usize)
                } else {
                    Err(format!("There is no value in memory at address {pointer_address}").into())
                }
            }
        }
    }
}

#[cfg(test)]
mod memory_tests {
    use super::*;

    #[test]
    fn test_memory_with_data() {
        let mut data = HashMap::new();
        data.insert(1, ValueBox::from(42));
        let memory = Memory::with_data(data, 10);

        assert_eq!(memory.data.get(&1), Some(&ValueBox::from(42)));
    }

    #[test]
    fn test_memory_can_set() {
        let memory = Memory::default();

        assert!(memory.can_set(&1));
        assert!(memory.can_set(&0));
        assert!(!memory.can_set(&11));
    }

    #[test]
    fn test_memory_set() {
        let mut memory = Memory::default();
        memory.set(&1, Some(ValueBox::from(42)));

        assert_eq!(memory.get(&1), Some(&ValueBox::from(42)));
    }

    #[test]
    fn test_memory_set_none() {
        let mut memory = Memory::default();
        memory.set(&1, Some(ValueBox::from(42)));
        memory.set(&1, None);

        assert_eq!(memory.get(&1), None);
    }

    #[test]
    #[should_panic]
    fn test_memory_set_out_of_bounds() {
        let mut memory = Memory::default();
        memory.max_address = 10;
        memory.set(&11, Some(ValueBox::from(42)));
    }
}
