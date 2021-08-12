use command_parser::{parser, CommandParse};

#[derive(PartialEq, Eq, Debug)]
pub struct MyInt(i32);

impl CommandParse for MyInt {
    fn parse_from_command(val: &str) -> Result<(&str, Self), &str> {
        let mut split = val.split(" ");
        let val = split.next().ok_or(val)?;
        let rest = split.next().unwrap_or("");
        let int = val.parse().map_err(|_err| rest)?;
        Ok((rest, MyInt(int)))
    }
}

impl std::fmt::Display for MyInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[parser]
mod my_module {
    #[derive(PartialEq, Eq, Debug)]
    pub enum Command {}

    #[parse("Foo Bar $baz")]
    #[derive(PartialEq, Eq, Debug)]
    pub struct Foo {
        baz: super::MyInt,
    }

    #[parse("Bar Baz $foo")]
    #[derive(PartialEq, Eq, Debug)]
    pub struct Bar {
        foo: super::MyInt
    }

    #[parse("This is a long command with option: $my_int", value=super::MyInt(0))]
    #[parse("Short version: $my_int $value")]
    #[derive(PartialEq, Eq, Debug)]
    pub struct MultipleOptions {
        pub value: super::MyInt,
        pub my_int: super::MyInt
    }
}

use my_module::Command;

#[test]
fn test_command_macro() {
    let foo: Command = "Foo Bar 42".parse().unwrap();
    assert_eq!(foo.to_string(), "Foo Bar 42");

    let bar: Command = "Bar Baz 150".parse().unwrap();
    assert_eq!(bar.to_string(), "Bar Baz 150");

    let multiple_options_a: Command = "This is a long command with option: 897".parse().unwrap();
    let multiple_options_b: Command = "Short version: 23874 15".parse().unwrap();
    assert_eq!(multiple_options_a, Command::MultipleOptions(my_module::MultipleOptions{value: MyInt(0), my_int: MyInt(897)}));
    assert_eq!(multiple_options_b, Command::MultipleOptions(my_module::MultipleOptions{value: MyInt(15), my_int: MyInt(23874)}));
    assert_eq!(multiple_options_a.to_string(), "This is a long command with option: 897");
    assert_eq!(multiple_options_b.to_string(), "Short version: 23874 15");
}
