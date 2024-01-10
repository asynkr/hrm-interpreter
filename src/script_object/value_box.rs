use std::str::FromStr;

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

#[derive(Debug, thiserror::Error)]
/// Error that can occur when parsing a ValueBox.
pub enum ParseValueBoxError {
    #[error("{0} is not a number nor a single character")]
    TooManyCharacters(String),
}

impl FromStr for ValueBox {
    type Err = ParseValueBoxError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: &str = &s.replace(' ', "");
        match s.parse::<i32>() {
            Ok(value) => Ok(Self::Number(value)),
            Err(_) if s.len() == 1 => {
                let c = s.chars().next().unwrap();
                Ok(Self::Character(c))
            }
            _ => Err(Self::Err::TooManyCharacters(s.to_string())),
        }
    }
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

#[derive(Debug, thiserror::Error)]
/// Error that can occur when parsing a "value box memory address".
pub enum ParseValueBoxMemoryAddressError {
    #[error("error parsing '{0}' as a pointer (should be a positive integer):\n\t{1}")]
    InvalidPointer(String, #[source] std::num::ParseIntError),
    #[error("error parsing '{0}' as a pointer address (should be a positive integer between brackets: [10]):\n\t{0}")]
    InvalidPointerAddress(String, #[source] std::num::ParseIntError),
}

impl FromStr for ValueBoxMemoryAddress {
    type Err = ParseValueBoxMemoryAddressError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // remove whitespaces characters
        let s: &str = &s.replace(' ', "");

        if s.starts_with('[') && s.ends_with(']') {
            let s_without_brackets = s.trim_start_matches('[').trim_end_matches(']').trim();

            s_without_brackets
                .parse::<usize>()
                .map(Self::PointerAddress)
                .map_err(|e| Self::Err::InvalidPointerAddress(s.to_string(), e))
        } else {
            s.parse::<usize>()
                .map(Self::Pointer)
                .map_err(|e| Self::Err::InvalidPointer(s.to_string(), e))
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
