# Command parser generator library
This library provides a single attribute-macro `parser` which can be used on a module to generate a parser for minecraft-like commands.

## Example
Simple parser for a `scoreboard players add/remove` command:

```rust
use command_parser::parser;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct ScoreboardPlayer {
    scoreboard: String,
    player: String,
}

impl fmt::Display for ScoreboardPlayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.player, self.scoreboard)
    }
}

impl command_parser::CommandParse for ScoreboardPlayer {
    fn parse_from_command(rest: &str) -> Result<(&str, Self), &str> {
        let (rest, player) = String::parse_from_command(rest)?;
        let (rest, scoreboard) = String::parse_from_command(rest)?;
        Ok((rest, ScoreboardPlayer { player, scoreboard }))
    }
}

#[parser]
mod commands {
    /// This enum will be populated with all defined parsers
    /// The target enum has to be the first item in the module!
    #[derive(Debug, PartialEq, Eq)]
    pub enum Command {}

    #[parse("scoreboard players add $target $value", add = true)]
    #[parse("scoreboard players remove $target $value", add = false)]
    #[derive(Debug, PartialEq, Eq)]
    pub struct ScoreboardAddImmediate {
        pub add: bool,
        pub target: super::ScoreboardPlayer,
        pub value: i32,
    }
}
use commands::Command;

fn main() {
    let cmd: Command = "scoreboard players add @a my_scoreboard 9000"
        .parse()
        .unwrap();
    assert_eq!(
        cmd,
        Command::ScoreboardAddImmediate(commands::ScoreboardAddImmediate {
            add: true,
            target: ScoreboardPlayer {
                scoreboard: "my_scoreboard".to_string(),
                player: "@a".to_string(),
            },
            value: 9000,
        })
    );
    println!("{}", cmd);

    let cmd2: Command = "scoreboard players remove @a my_scoreboard 9001"
        .parse()
        .unwrap();
    assert_eq!(
        cmd2,
        Command::ScoreboardAddImmediate(commands::ScoreboardAddImmediate {
            add: false,
            target: ScoreboardPlayer {
                scoreboard: "my_scoreboard".to_string(),
                player: "@a".to_string(),
            },
            value: 9001,
        })
    );
    println!("{}", cmd2);
}

```

Output:
```
scoreboard players add @a my_scoreboard 9000
scoreboard players remove @a my_scoreboard 9001
```

The code gets expanded into roughly this:
```rust
mod commands {
    /// This enum will be populated with all defined parsers
    /// The target enum has to be the first item in the module!
    #[derive(Debug, PartialEq, Eq)]
    pub enum Command {
        ScoreboardAddImmediate(ScoreboardAddImmediate),
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct ScoreboardAddImmediate {
        pub add: bool,
        pub target: super::ScoreboardPlayer,
        pub value: i32,
    }

    impl fmt::Display for Command {
        // ...
    }

    impl From<ScoreboardAddImmediate> for Command {
        // ...
    }

    impl fmt::Display for ScoreboardAddImmediate {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match (&self.add,) {
                (true,) => write!(f, "scoreboard players add {} {}", &self.target, &self.value),
                (false,) => write!(
                    f,
                    "scoreboard players remove {} {}",
                    &self.target, &self.value
                ),
                _ => unreachable!(
                    "Cannot convert invalid struct to string: Does not respect parsing invariants"
                ),
            }
        }
    }

    impl FromStr for Command {
        type Err = String;

        fn from_str(s: &str) -> Result<Self, String> {
            fn parse(rest: &str) -> Result<Command, &str> {
                let (next, rest) = rest.split_once(" ").unwrap_or((rest, ""));
                match next {
                    "scoreboard" => {
                        let (next, rest) = rest.split_once(" ").unwrap_or((rest, ""));
                        match next {
                            "players" => {
                                let (next, rest) = rest.split_once(" ").unwrap_or((rest, ""));
                                match next {
                                    "add" => {
                                        if let Ok((rest, _target)) =  <super::ScoreboardPlayer as command_parser::CommandParse>::parse_from_command(rest){
                                            if let Ok((rest, _value)) =  <i32 as command_parser::CommandParse>::parse_from_command(rest){
                                                if rest.is_empty() {
                                                    let _add = true;
                                                    return Ok(ScoreboardAddImmediate {
                                                        add:_add,value:_value,target:_target
                                                    }.into())
                                                } else {
                                                    return Err(rest);
                                                }
                                            } else {
                                                return Err(rest)
                                            }
                                        } else {
                                            return Err(rest)
                                        }
                                    },
                                    "remove" => {
                                        if let Ok((rest, _target)) =  <super::ScoreboardPlayer as command_parser::CommandParse>::parse_from_command(rest) {
                                            if let Ok((rest, _value)) =  <i32 as command_parser::CommandParse>::parse_from_command(rest) {
                                                if rest.is_empty(){
                                                    let _add = false;
                                                    return Ok(ScoreboardAddImmediate {
                                                        add:_add,value:_value,target:_target
                                                    }.into())
                                                } else {
                                                    return Err(rest);
                                                }
                                            } else {
                                                return Err(rest)
                                            }
                                        } else {
                                            return Err(rest)
                                        }
                                    }
                                    _ => return Err(rest),
                                }
                            }
                            _ => return Err(rest),
                        }
                    }
                    _ => return Err(rest),
                }
            }
            parse(s).map_err(::std::string::String::from)
        }
    }
}
```