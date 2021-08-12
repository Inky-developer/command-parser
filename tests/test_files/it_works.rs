use command_parser::parser;

#[parser]
mod my_module {
    enum Command {}

    #[parse("Foo")]
    struct HelloWorld {}

    #[parse("Bar")]
    struct Bar {}
}

fn main() {}