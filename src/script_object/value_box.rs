#[derive(Clone)]
/// Wrapper for a value that can be stored in memory.
pub enum ValueBox {
    Number(i32),
    Character(char),
}

/// Wrapper for a memory address.
/// Can be either a direct memory address or a pointer at which the memory address is stored.
pub enum ValueBoxMemAddress {
    Pointer(u32),
    PointerAddress(u32),
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