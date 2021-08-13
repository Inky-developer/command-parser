extern crate parse_macro;

pub use parse_macro::parser;

pub trait CommandParse: std::fmt::Display {
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str>
    where
        Self: Sized;
}

pub fn parse_command<T>(value: &str) -> Result<T, &str>
where
    T: CommandParse,
{
    let (rest, value) = T::parse_from_command(value)?;
    if !rest.is_empty() {
        Err(rest)
    } else {
        Ok(value)
    }
}

/// Parses a single word and returns a tuple of `(rest, word)`
pub fn parse_str(value: &str) -> (&str, &str) {
    let (value, rest) = value.split_once(" ").unwrap_or((value, ""));
    (rest, value)
}

impl CommandParse for i32 {
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str> {
        let (rest, value_str) = parse_str(value);
        let value = value_str.parse().map_err(|_| rest)?;
        Ok((rest, value))
    }
}

impl CommandParse for String {
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str> {
        let (rest, value) = parse_str(value);
        Ok((rest, value.to_string()))
    }
}
