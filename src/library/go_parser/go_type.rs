use crate::library::lexeme::definition::TokenType;

#[derive(Copy, Clone, Debug)]
pub enum Modifier {
    Unsigned,
    Signed,
    Const,
    Static,
    Default, //none applied
}

/// Takes the integer value of type Type
/// returns either the equivalent Rust type as a string or
/// None, if does not correspond to any c/c++ type
pub fn parse_type(c_type: TokenType, modifier: Modifier) -> Option<String> {
    match (modifier, c_type) {
        //unsigned types
        (Modifier::Unsigned, TokenType::Character) => Some("byte".to_string()),
        (Modifier::Unsigned, TokenType::Short) => Some("uint16".to_string()),
        (Modifier::Unsigned, TokenType::Integer) => Some("uint32".to_string()),
        (Modifier::Unsigned, TokenType::Long) => Some("uint64".to_string()),

        //signed types
        (_, TokenType::Short) => Some("int16".to_string()),
        (_, TokenType::Integer) => Some("int".to_string()),
        (_, TokenType::Long) => Some("int64".to_string()),

        //type without modifiers
        (_, TokenType::Float) => Some("float32".to_string()),
        (_, TokenType::Double) => Some("float64".to_string()),
        (_, TokenType::Character) => Some("byte".to_string()),
        (_, TokenType::Boolean) => Some("bool".to_string()),
        (_, TokenType::Void) => Some("void".to_string()),
        (_, TokenType::Auto) => Some("_".to_string()),
        (_, TokenType::StringValue) => Some("string".to_string()),
        (_, _) => None,
    }
}

pub fn get_default_value_for(c_type: TokenType) -> String {
    let value = match c_type {
        TokenType::Integer => "0",
        TokenType::Short => "int16(0)",
        TokenType::Long => "int64(0)",
        TokenType::Float => "float32(0)",
        TokenType::Double => "float64(0)",
        TokenType::Character => "'_'",
        TokenType::Boolean => "false",
        _ => "_",
    };
    String::from(value)
}
