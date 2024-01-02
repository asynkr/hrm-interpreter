use std::{error::Error, str::FromStr};

#[derive(Copy, Clone, PartialEq, Debug)]
/// Wrapper for a value that can be stored in memory.
/// The name comes from the fact that in HRM, the values are like cardboard boxes.
/// A ValueBox can be either a number or a character.
pub enum ValueBox {
    Number(i32),
    Character(char),
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// Wrapper for a memory address.
/// It can be either a direct memory address or a pointer at which the memory address is stored.
///
/// Ex:
/// - "Copy from 2" uses Pointer(2)
/// and means "Copy from the value at memory address 2"
/// - "Copy from \[2]" uses PointerAddress(2)
/// and means "Copy from the value at the memory address stored at memory address 2",
/// ie "Read the value at memory address 2, and use it as a memory address to read the desired value from"
pub enum ValueBoxMemoryAddress {
    Pointer(usize),
    PointerAddress(usize),
}

impl From<i32> for ValueBox {
    fn from(value: i32) -> Self {
        Self::Number(value)
    }
}

impl From<char> for ValueBox {
    fn from(value: char) -> Self {
        Self::Character(value)
    }
}

impl ToString for ValueBox {
    fn to_string(&self) -> String {
        match self {
            Self::Number(value) => value.to_string(),
            Self::Character(value) => value.to_string(),
        }
    }
}

impl FromStr for ValueBoxMemoryAddress {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // remove whitespaces characters
        let s: &str = &s.replace(' ', "");

        if s.starts_with('[') && s.ends_with(']') {
            let s = s.trim_start_matches('[').trim_end_matches(']');
            let s = s.trim();

            match s.parse::<usize>() {
                Ok(address) => Ok(Self::PointerAddress(address)),
                Err(_) => Err("Invalid memory address".into()),
            }
        } else {
            let address = s.parse::<usize>()?;

            Ok(Self::Pointer(address))
        }
    }
}

#[cfg(test)]
mod value_box_tests {
    use super::*;

    #[test]
    fn test_value_box_from_i32() {
        let value = ValueBox::from(42);

        assert_eq!(value, ValueBox::Number(42));
    }

    #[test]
    fn test_value_box_from_char() {
        let value = ValueBox::from('a');

        assert_eq!(value, ValueBox::Character('a'));
    }

    #[test]
    fn test_value_box_to_string() {
        let value = ValueBox::from(42);

        assert_eq!(value.to_string(), "42");
    }
}

#[cfg(test)]
mod vbma_tests {
    use super::*;

    #[test]
    fn test_value_box_mem_address_from_str() {
        let address = ValueBoxMemoryAddress::from_str("42").unwrap();

        assert_eq!(address, ValueBoxMemoryAddress::Pointer(42));
    }

    #[test]
    fn test_value_box_mem_address_from_str_with_brackets() {
        let address = ValueBoxMemoryAddress::from_str("[42]").unwrap();

        assert_eq!(address, ValueBoxMemoryAddress::PointerAddress(42));
    }

    #[test]
    fn test_value_box_mem_address_from_str_with_brackets_and_spaces() {
        let address = ValueBoxMemoryAddress::from_str("[ 42  ]").unwrap();

        assert_eq!(address, ValueBoxMemoryAddress::PointerAddress(42));
    }

    #[test]
    #[should_panic]
    fn test_value_box_mem_address_from_str_with_invalid_address() {
        let _address = ValueBoxMemoryAddress::from_str("invalid").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_value_box_mem_address_from_str_with_negative_number() {
        let _address = ValueBoxMemoryAddress::from_str("[-25]").unwrap();
    }
}
