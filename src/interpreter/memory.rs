use std::collections::HashMap;

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

    pub fn get_max_address(&self) -> usize {
        self.max_address
    }
}

#[derive(Debug, thiserror::Error)]
/// Error that can occur when reading a value and assuming it's not None.
pub enum GetMemoryError {
    #[error("no value at address {0} given by {1:?}")]
    NoValueAtAddress(usize, ValueBoxMemoryAddress),
    #[error("invalid value box memory address:\n\t{0}")]
    InvalidValueBoxMemoryAddress(#[from] ReadValueBoxMemoryAddressError),
}

#[derive(Debug, thiserror::Error)]
/// Error that can occur when setting a value.
pub enum SetMemoryError {
    #[error("Memory address {address} out of bounds (accepted: [1, {max_address}])")]
    OutOfBounds { address: usize, max_address: usize },
    #[error("invalid value box memory address:\n\t{0}")]
    InvalidValueBoxMemoryAddress(#[from] ReadValueBoxMemoryAddressError),
}

// General methods
impl Memory {
    /// at_adress is at most max_len - 1
    pub fn is_valid_memory_address(&self, at_address: &usize) -> bool {
        at_address <= &self.max_address
    }

    /// Get the value at the given address.
    pub fn get(&self, address: &usize) -> Option<&ValueBox> {
        self.data.get(address)
    }

    /// Get the value at the given "value box memory address",
    /// or return an error if there is no value at this address,
    /// or if the address is invalid.
    pub fn get_with_vbma(&self, vbma: &ValueBoxMemoryAddress) -> Result<&ValueBox, GetMemoryError> {
        let address = self.translate_vbma_to_mem_address(vbma)?;
        self.get(&address)
            .ok_or(GetMemoryError::NoValueAtAddress(address, *vbma))
    }

    /// Set the value at the given address.
    pub fn set(&mut self, address: &usize, value: Option<ValueBox>) -> Result<(), SetMemoryError> {
        if !self.is_valid_memory_address(address) {
            // address is bound by max_len
            return Err(SetMemoryError::OutOfBounds {
                address: *address,
                max_address: self.max_address,
            });
        }

        match value {
            Some(value) => {
                self.data.insert(*address, value);
            }
            None => {
                self.data.remove(address);
            }
        }
        Ok(())
    }

    /// Set the value at the given "value box memory address"
    pub fn set_with_vbma(
        &mut self,
        vbma: &ValueBoxMemoryAddress,
        value: Option<ValueBox>,
    ) -> Result<(), SetMemoryError> {
        let address = self.translate_vbma_to_mem_address(vbma)?;
        self.set(&address, value)
    }
}

#[derive(Debug, thiserror::Error)]
/// Error that can occur when decoding a "value box memory address".
pub enum ReadValueBoxMemoryAddressError {
    #[error("Value {value_tested} in memory at {pointer_address} is negative, which is not a valid memory address")]
    NegativePointerAddress {
        value_tested: i32,
        pointer_address: usize,
    },
    #[error(
        "There is no value in memory at address {0} to be interpreted as a memory address itself (given by {1:?})"
    )]
    NoValueAtAddress(usize, ValueBoxMemoryAddress),
    #[error("final address {final_address} given by {vbma:?} is out of bounds (accepted: [0, {max_address}])")]
    OutOfBounds {
        final_address: usize,
        vbma: ValueBoxMemoryAddress,
        max_address: usize,
    },
}

// Specific methods
impl Memory {
    /// Translate a "value box memory address" to a memory address.
    /// It can be a direct memory address, or a pointer to a memory address.
    /// In both cases, the validity of the memory address is checked.
    pub fn translate_vbma_to_mem_address(
        &self,
        value_box_memory_address: &ValueBoxMemoryAddress,
    ) -> Result<usize, ReadValueBoxMemoryAddressError> {
        let final_address = match value_box_memory_address {
            // VBMA is a direct memory address
            ValueBoxMemoryAddress::Pointer(address) => *address,
            // VBMA is a pointer to a memory address
            ValueBoxMemoryAddress::PointerAddress(pointer_address) => {
                match self.get(pointer_address) {
                    Some(ValueBox::Number(address)) => {
                        if *address < 0 {
                            return Err(ReadValueBoxMemoryAddressError::NegativePointerAddress {
                                value_tested: *address,
                                pointer_address: *pointer_address,
                            });
                        }
                        *address as usize
                    }
                    _ => {
                        return Err(ReadValueBoxMemoryAddressError::NoValueAtAddress(
                            *pointer_address,
                            *value_box_memory_address,
                        ))
                    }
                }
            }
        };

        if !self.is_valid_memory_address(&final_address) {
            return Err(ReadValueBoxMemoryAddressError::OutOfBounds {
                final_address,
                vbma: *value_box_memory_address,
                max_address: self.max_address,
            });
        }
        Ok(final_address)
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
        let mut memory = Memory::default();
        memory.max_address = 10;

        assert!(memory.is_valid_memory_address(&1));
        assert!(memory.is_valid_memory_address(&0));
        assert!(!memory.is_valid_memory_address(&11));
    }

    #[test]
    fn test_memory_set() {
        let mut memory = Memory::default();
        memory.set(&1, Some(ValueBox::from(42))).unwrap();

        assert_eq!(memory.get(&1), Some(&ValueBox::from(42)));
    }

    #[test]
    fn test_memory_set_none() {
        let mut memory = Memory::default();
        memory.set(&1, Some(ValueBox::from(42))).unwrap();
        memory.set(&1, None).unwrap();

        assert_eq!(memory.get(&1), None);
    }

    #[test]
    #[should_panic]
    fn test_memory_set_out_of_bounds() {
        let mut memory = Memory::default();
        memory.max_address = 10;
        memory.set(&11, Some(ValueBox::from(42))).unwrap();
    }
}
