use std::collections::HashMap;

use proc_macro2::Ident;
use syn::{Expr, Type};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseTree {
    pub payload: ParseNode,
    pub options: Vec<ParseTree>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseNode {
    EndOfInput {
        struct_name: Ident,
        defaults: HashMap<Ident, Expr>,
        idents: Vec<Ident>,
    },
    Pass,
    Literal(String),
    Function {
        name: Type,
        binding: Ident,
    },
}

impl ParseTree {
    pub fn new() -> Self {
        ParseTree {
            options: Vec::default(),
            payload: ParseNode::Pass,
        }
    }

    pub fn insert(
        &mut self,
        items: impl Iterator<Item = ParseNode>,
        struct_name: Ident,
        defaults: HashMap<Ident, Expr>,
        idents: Vec<Ident>,
    ) {
        let with_end_of_input = items.chain(std::iter::once(ParseNode::EndOfInput {
            defaults,
            struct_name,
            idents,
        }));
        let mut tree = self;

        for item in with_end_of_input {
            let is_in_tree = tree.options.iter().any(|tree| tree.payload == item);
            if is_in_tree {
                tree = tree
                    .options
                    .iter_mut()
                    .find(|tree| tree.payload == item)
                    .unwrap();
            } else {
                tree.options.push(ParseTree {
                    payload: item,
                    options: Default::default(),
                });
                tree = tree.options.last_mut().unwrap();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use proc_macro2::{Ident, Span};

    use crate::parse_tree::ParseNode;

    use super::ParseTree;

    #[test]
    fn test_build_tree() {
        let defaults = HashMap::new();
        let idents = Vec::new();
        let struct_name = Ident::new("Foo", Span::call_site());

        let mut tree = ParseTree::new();

        tree.insert(
            vec![
                ParseNode::Literal("execute".to_string()),
                ParseNode::Literal("store".to_string()),
                ParseNode::Literal("foo".to_string()),
            ]
            .into_iter(),
            struct_name.clone(),
            defaults.clone(),
            idents.clone(),
        );

        tree.insert(
            vec![
                ParseNode::Literal("execute".to_string()),
                ParseNode::Literal("store".to_string()),
                ParseNode::Literal("bar".to_string()),
            ]
            .into_iter(),
            struct_name.clone(),
            defaults.clone(),
            idents.clone(),
        );

        assert_eq!(
            tree.options,
            vec![ParseTree {
                options: vec![ParseTree {
                    options: vec![
                        ParseTree {
                            options: vec![ParseTree {
                                options: vec![],
                                payload: ParseNode::EndOfInput {
                                    struct_name: struct_name.clone(),
                                    defaults: defaults.clone(),
                                    idents: idents.clone(),
                                },
                            }],
                            payload: ParseNode::Literal("foo".to_string())
                        },
                        ParseTree {
                            options: vec![ParseTree {
                                options: vec![],
                                payload: ParseNode::EndOfInput {
                                    struct_name: struct_name.clone(),
                                    defaults: defaults.clone(),
                                    idents: idents.clone(),
                                }
                            }],
                            payload: ParseNode::Literal("bar".to_string()),
                        }
                    ],
                    payload: ParseNode::Literal("store".to_string())
                }],
                payload: ParseNode::Literal("execute".to_string())
            }]
        )
    }
}
