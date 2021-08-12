extern crate parse_macro;

pub use parse_macro::parser;

pub trait CommandParse: std::fmt::Display {
    fn parse_from_command(value: &str) -> Result<(&str, Self), &str>
    where
        Self: Sized;
}
