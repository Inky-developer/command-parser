use command_parser::parser;

struct MyInt(i32);

impl command_parser::CommandParse for MyInt {
    fn parse_from_command(val: &str) -> Result<(&str, Self), &str> {
        let (maybe_int, rest) = val.split_once(" ").ok_or(val)?;
        let int = maybe_int.parse().map_err(|_err| rest)?;
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
    enum Command {}

    #[parse("Foo Bar $baz")]
    struct HelloWorld {
        baz: super::MyInt
    }
}

fn main() {}