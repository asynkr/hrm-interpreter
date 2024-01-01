use std::{error::Error, str::FromStr};

#[derive(Copy, Clone, PartialEq, Debug)]
/// Wrapper for a value that can be stored in memory.
pub enum ValueBox {
    Number(i32),
    Character(char),
}

#[derive(Copy, Clone, PartialEq, Debug)]
/// Wrapper for a memory address.
/// Can be either a direct memory address or a pointer at which the memory address is stored.
pub enum ValueBoxMemAddress {
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

impl FromStr for ValueBoxMemAddress {
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

impl ValueBoxMemAddress {
    pub fn get_address(&self, memory: &[Option<ValueBox>]) -> Result<usize, Box<dyn Error>> {
        match self {
            Self::Pointer(address) => Ok(*address),
            Self::PointerAddress(pointer_address) => {
                let address = memory.get(*pointer_address);
                if let Some(Some(ValueBox::Number(n))) = address {
                    if n < &0 {
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
        let address = ValueBoxMemAddress::from_str("42").unwrap();

        assert_eq!(address, ValueBoxMemAddress::Pointer(42));
    }

    #[test]
    fn test_value_box_mem_address_from_str_with_brackets() {
        let address = ValueBoxMemAddress::from_str("[42]").unwrap();

        assert_eq!(address, ValueBoxMemAddress::PointerAddress(42));
    }

    #[test]
    fn test_value_box_mem_address_from_str_with_brackets_and_spaces() {
        let address = ValueBoxMemAddress::from_str("[ 42  ]").unwrap();

        assert_eq!(address, ValueBoxMemAddress::PointerAddress(42));
    }

    #[test]
    #[should_panic]
    fn test_value_box_mem_address_from_str_with_invalid_address() {
        let _address = ValueBoxMemAddress::from_str("invalid").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_value_box_mem_address_from_str_with_negative_number() {
        let _address = ValueBoxMemAddress::from_str("[-25]").unwrap();
    }

    #[test]
    fn test_value_box_mem_address_get_address() {
        let memory = vec![Some(ValueBox::Number(42)), Some(ValueBox::Number(1))];

        let vbma = ValueBoxMemAddress::Pointer(0);
        let address = vbma.get_address(&memory).unwrap();

        assert_eq!(address, 0);
    }

    #[test]
    fn test_value_box_mem_pointer_address_get_address() {
        let memory = vec![Some(ValueBox::Number(42)), Some(ValueBox::Number(1))];

        let vbma = ValueBoxMemAddress::PointerAddress(1);
        let address = vbma.get_address(&memory).unwrap();

        assert_eq!(address, 1);
    }
}
