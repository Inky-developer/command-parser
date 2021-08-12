use command_parser::parser;

#[parser]
mod my_module {
    enum Command {}

    #[parse("Foo Bar", came_from=true)]
    #[parse("Foo Baz", came_from=false)]
    struct HelloWorld {
        came_from: bool
    }
}

fn main() {}