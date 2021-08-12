use command_parser::parser;

#[parser]
mod my_module {
    enum Command {}

    #[parse("Foo Bar Baz")]
    #[parse("Foo Bar Qux")]
    struct HelloWorld {}
}

fn main() {}