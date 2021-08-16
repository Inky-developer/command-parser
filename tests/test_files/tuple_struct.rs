use command_parser::parser;

#[parser]
mod my_module {
    enum Command {}

    #[parse("Foo $0")]
    struct HelloWorld(String);
}

fn main() {}