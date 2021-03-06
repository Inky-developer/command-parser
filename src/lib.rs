extern crate parse_macro;

pub use parse_macro::parser;

pub trait CommandParse: std::fmt::Display + Sized {
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str>;
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

/// Parse the string as T or as none if it could not be parsed
pub fn parse_optional_command<T>(value: &str) -> (&str, Option<T>)
where
    T: CommandParse,
{
    match T::parse_from_command(value) {
        Ok((rest, value)) => (rest, Some(value)),
        Err(_) => (value, None),
    }
}

/// Parses `T` repeatedly as long as possible
pub fn parse_multiple_commands<T>(value: &str) -> (&str, Vec<T>)
where
    T: CommandParse,
{
    let mut items = Vec::new();
    let mut rest = value;
    while let Ok((next_rest, item)) = T::parse_from_command(rest) {
        rest = next_rest;
        items.push(item)
    }
    (rest, items)
}

/// Parses a single word and returns a tuple of `(rest, word)`
pub fn parse_str(value: &str) -> (&str, &str) {
    let (value, rest) = value.split_once(" ").unwrap_or((value, ""));
    (rest, value)
}

impl CommandParse for i32 {
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str> {
        let end_idx = if let Some(value_neg) = value.strip_prefix('-') {
            value_neg
                .find(|c: char| !c.is_digit(10))
                .unwrap_or(value_neg.len())
                + 1
        } else {
            value.find(|c: char| !c.is_digit(10)).unwrap_or(value.len())
        };

        let (value_str, rest) = value.split_at(end_idx);
        let value = value_str.parse().map_err(|_| value)?;
        Ok((rest, value))
    }
}

impl CommandParse for f32 {
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str> {
        // TODO: This will break on comma-separated lists
        let (rest, value_str) = parse_str(value);
        let value = value_str.parse().map_err(|_| value)?;
        Ok((rest, value))
    }
}

impl CommandParse for f64 {
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str> {
        // TODO: This will break on comma-separated lists
        let (rest, value_str) = parse_str(value);
        let value = value_str.parse().map_err(|_| value)?;
        Ok((rest, value))
    }
}

impl CommandParse for String {
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str> {
        let (rest, value) = parse_str(value);
        if value.is_empty() {
            return Err(rest);
        }
        Ok((rest, value.to_string()))
    }
}

impl<T> CommandParse for Box<T>
where
    T: CommandParse,
{
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str> {
        let (rest, value) = T::parse_from_command(value)?;
        Ok((rest, Box::new(value)))
    }
}

#[cfg(test)]
mod test {
    use super::CommandParse;

    #[test]
    fn test_i32() {
        assert_eq!(i32::parse_from_command("123"), Ok(("", 123)));
        assert_eq!(i32::parse_from_command("-123"), Ok(("", -123)));
        assert_eq!(i32::parse_from_command("17 34"), Ok((" 34", 17)));
        assert_eq!(i32::parse_from_command("17, 34"), Ok((", 34", 17)));
        assert_eq!(i32::parse_from_command("abcd 123"), Err("abcd 123"));
    }
}
