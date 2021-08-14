mod parse_tree;

extern crate proc_macro;
use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use parse_tree::ParseNode;
use proc_macro::TokenStream;

use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned,
    Attribute, AttributeArgs, Expr, Fields, Item, ItemEnum, ItemMod, ItemStruct, LitStr, Token,
    Type,
};

use crate::parse_tree::ParseTree;

#[proc_macro_attribute]
pub fn parser(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemMod);
    let args = parse_macro_input!(args as AttributeArgs);

    let result = handle_parse_macro(args, input);
    let result = match result {
        Ok(data) => data.into(),
        Err(error) => {
            let e = error.to_compile_error();
            quote! { #e }
        }
    };

    result.into()
}

fn handle_parse_macro(
    args: AttributeArgs,
    mut input: ItemMod,
) -> syn::Result<proc_macro2::TokenStream> {
    if args.len() > 0 {
        return Err(syn::Error::new(args[0].span(), "Expected no arguments"));
    }

    let config = MacroConfig::new(&mut input)?;
    let mut parse_tree = ParseTree::new();

    let mut structs = Vec::new();
    let mut display_impls: Vec<Item> = Vec::new();

    if let Some((_brace, content)) = input.content.as_mut() {
        for item in content {
            if let Item::Struct(strukt) = item {
                let fields = extract_struct_fields(&strukt)?;
                let mut has_attr = false;

                let mut optional_args = HashSet::new();

                let interesting_attributes =
                    find_interesting_attributes(&mut strukt.attrs, &fields)?;
                for matching_attribute in &interesting_attributes {
                    has_attr = true;
                    parse_tree.insert(
                        matching_attribute.parse_template.iter().cloned(),
                        strukt.ident.clone(),
                        matching_attribute.kwargs.clone(),
                        fields.keys().cloned().collect(),
                    );

                    optional_args.extend(matching_attribute.kwargs.keys().cloned());
                }

                if has_attr {
                    let optional_args = optional_args.into_iter().collect();
                    let display_impl =
                        generate_display_impl(optional_args, &interesting_attributes, &strukt);
                    display_impls.push(display_impl);
                    structs.push(strukt.ident.clone());
                }
            }
        }
    }

    let target_enum = find_target_enum(&mut input)?;
    if !target_enum.variants.is_empty() {
        return Err(syn::Error::new(
            target_enum.span(),
            "The target enum must be empty",
        ));
    }
    for strukt in &structs {
        target_enum.variants.push(parse_quote! {
            #strukt(#strukt)
        })
    }

    if let Some((_brace, content)) = input.content.as_mut() {
        let enum_name = &config.output_name;
        content.push(parse_quote! {
            impl ::std::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    match self {#(
                        Self::#structs(val) => write!(f, "{}", val)
                    ),*}
                }
            }
        });

        for strukt in &structs {
            content.push(parse_quote! {
                impl ::std::convert::From<#strukt> for #enum_name {
                    fn from(value: #strukt) -> Self {
                        Self::#strukt(value)
                    }
                }
            });
        }

        content.extend(display_impls.into_iter());

        let from_string_impl = generate_from_string_impl(&parse_tree, &config.output_name);
        content.push(from_string_impl)
    }

    Ok(input.to_token_stream())
}

fn generate_display_impl(
    optional_args: Vec<Ident>,
    interesting_attributes: &[ParseAttr],
    strukt: &ItemStruct,
) -> Item {
    let mut pattern_matches = Vec::new();
    let mut write_actions = Vec::new();
    for matching_attribute in interesting_attributes {
        let values = optional_args
            .iter()
            .map(|arg| match matching_attribute.kwargs.get(arg) {
                Some(expr) => quote! {#expr},
                None => quote! {_},
            });
        let pattern_match = quote! {
            (#(#values),*)
        };
        pattern_matches.push(pattern_match);

        let mut arg_assignments = Vec::new();
        let mut template_parts = Vec::new();
        for template_part in &matching_attribute.parse_template {
            match template_part {
                ParseNode::Pass | ParseNode::EndOfInput { .. } => unreachable!(),
                ParseNode::Function { name: _, binding } => {
                    arg_assignments.push(quote! {&self.#binding});
                    template_parts.push("{}")
                }
                ParseNode::Literal(val) => {
                    template_parts.push(&val);
                }
            };
        }

        let template_parts = template_parts.into_iter().join(" ");
        let write_action = quote! {
            write!(f, #template_parts, #(#arg_assignments),*)
        };
        write_actions.push(write_action);
    }
    let optional_args = quote! {
        (#(
            &self.#optional_args
        ),*)
    };
    let struct_name = &strukt.ident;
    let val: Item = parse_quote! {
        impl ::std::fmt::Display for #struct_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match #optional_args {
                    #(
                        #pattern_matches => #write_actions
                    ),*,
                    _ => unreachable!("Cannot convert invalid struct to string: Does not respect parsing invariants")
                }
            }
        }
    };
    val
}

fn generate_from_string_impl(parse_tree: &ParseTree, enum_name: &Ident) -> Item {
    match &parse_tree.payload {
        ParseNode::Pass => {
            let ts = _generate_from_string_impl_inner(&parse_tree.options);
            parse_quote! {
                impl ::std::str::FromStr for #enum_name {
                    type Err = ::std::string::String;

                    fn from_str(s: &::std::primitive::str) -> ::std::result::Result<Self, ::std::string::String> {
                        fn parse(rest: &::std::primitive::str) -> ::std::result::Result<#enum_name, &::std::primitive::str> {
                            #ts
                        }
                        parse(s).map_err(::std::string::String::from)
                    }
                }
            }
        }
        _ => panic!("Expected a tree root, with payload `Pass`"),
    }
}

fn _generate_from_string_impl_inner(options: &[ParseTree]) -> proc_macro2::TokenStream {
    let mut literal_matches: Vec<&str> = Vec::new();
    let mut literal_matches_and_then = Vec::new();

    let mut stop_matching = None;

    let mut function_matches_name = Vec::new();
    let mut function_matches_binding = Vec::new();
    let mut function_matches_and_then = Vec::new();

    for option in options {
        match &option.payload {
            ParseNode::Literal(lit) => {
                literal_matches.push(&lit);
                literal_matches_and_then.push(_generate_from_string_impl_inner(&option.options));
            }
            ParseNode::Function { name, binding } => {
                function_matches_name.push(name);
                function_matches_binding.push(binding);
                function_matches_and_then.push(_generate_from_string_impl_inner(&option.options));
            }
            ParseNode::EndOfInput {
                defaults,
                struct_name,
                idents,
            } => stop_matching = Some((defaults, struct_name, idents)),
            ParseNode::Pass => panic!("Invalid node: Pass"),
        }
    }

    fn escape_ident(ident: &Ident) -> Ident {
        Ident::new(&format!("_{}", ident.to_string()), ident.span())
    }

    let match_on_stop = if let Some((defaults, struct_name, idents)) = stop_matching {
        let escaped_idents = idents.iter().map(escape_ident);
        let escaped_default_idents = defaults.keys().map(escape_ident);
        let default_values = defaults.values();
        quote! {
            if rest.is_empty() {
                #(
                    let #escaped_default_idents = #default_values;
                )*
                return ::std::result::Result::Ok(#struct_name {#(
                    #idents: #escaped_idents
                ),*}.into())
            } else {
                return ::std::result::Result::Err(rest);
            }
        }
    } else {
        quote! {
            return ::std::result::Result::Err(rest)
        }
    };
    let match_on_function = if !function_matches_binding.is_empty() {
        let function_matches_binding_escaped = function_matches_binding
            .iter()
            .map(|ident| escape_ident(*ident));
        quote! {
            #(
                if let Ok((rest, #function_matches_binding_escaped)) = <#function_matches_name as ::command_parser::CommandParse>::parse_from_command(rest) {
                    #function_matches_and_then
                }
            )else*
            else {
                #match_on_stop
            }
        }
    } else {
        quote! {
            #match_on_stop
        }
    };
    let match_on_literal = if !literal_matches.is_empty() {
        quote! {
            let (next, rest) = rest.split_once(" ").unwrap_or((rest, ""));
            match next {
                #(
                    #literal_matches => {#literal_matches_and_then}
                ),*
                _ => #match_on_function,
            }
        }
    } else {
        quote! {
            #match_on_function
        }
    };

    match_on_literal
}

fn extract_struct_fields(strukt: &ItemStruct) -> syn::Result<HashMap<Ident, Type>> {
    let mut res = HashMap::new();
    match &strukt.fields {
        Fields::Named(named) => {
            for field in &named.named {
                res.insert(field.ident.as_ref().unwrap().clone(), field.ty.clone());
            }
        }
        Fields::Unnamed(_) => {
            return Err(syn::Error::new(
                strukt.span(),
                "Tuple structs not implemented for now. Use a regular struct instead.",
            ));
        }
        Fields::Unit => {}
    }

    Ok(res)
}

fn find_interesting_attributes(
    attrs: &mut Vec<Attribute>,
    fields: &HashMap<Ident, Type>,
) -> syn::Result<Vec<ParseAttr>> {
    let mut other_attributes = Vec::with_capacity(attrs.len());
    let mut found_attributes = Vec::new();
    for attr in attrs.drain(..) {
        if attr.path.is_ident("parse") {
            let attr_data = attr.parse_args_with(AttributeData::parse)?;
            let parse_attrs = ParseAttr::new(attr_data, &fields)?;
            found_attributes.push(parse_attrs);
        } else {
            other_attributes.push(attr);
        }
    }
    *attrs = other_attributes;
    Ok(found_attributes)
}

#[derive(Debug)]
struct ParseAttr {
    parse_template: Vec<ParseNode>,
    kwargs: HashMap<Ident, Expr>,
}

impl ParseAttr {
    pub fn new(attr_data: AttributeData, fields: &HashMap<Ident, Type>) -> syn::Result<Self> {
        let mut parse_template = Vec::new();
        for part in attr_data.parse_template.value().split_ascii_whitespace() {
            let parse_node = match part.strip_prefix('$') {
                Some(var) => ParseNode::Function {
                    binding: Ident::new(var, attr_data.parse_template.span()),
                    name: fields
                        .get(&Ident::new(var, attr_data.parse_template.span()))
                        .ok_or_else(|| {
                            syn::Error::new(
                                attr_data.parse_template.span(),
                                format!("Could not find '{}' in this struct", var),
                            )
                        })?
                        .clone(),
                },
                None => ParseNode::Literal(part.to_string()),
            };
            parse_template.push(parse_node);
        }

        let mut defaults = HashMap::new();
        for kwarg in attr_data.defaults {
            defaults.insert(kwarg.keyword, kwarg.value);
        }

        Ok(ParseAttr {
            kwargs: defaults,
            parse_template,
        })
    }
}

/// Finds the target enum, which is the first enum in the module
fn find_target_enum(module: &mut ItemMod) -> syn::Result<&mut ItemEnum> {
    let span = module.span();
    module.content.as_mut().and_then(|(_, content)| {
        content.iter_mut().find_map(|item| {
            if let Item::Enum(val) = item {
                Some(val)
            } else {
                None
            }
        })
    }).ok_or_else(|| syn::Error::new(span, "Could not find target enum. The target enum has to be the first enum in the module"))
}

struct AttributeData {
    parse_template: LitStr,
    defaults: Punctuated<KeywordArg, Token![,]>,
}

impl Parse for AttributeData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parse_template = input.parse()?;

        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
        }
        let defaults = input.parse_terminated(KeywordArg::parse)?;

        Ok(AttributeData {
            defaults,
            parse_template,
        })
    }
}

struct KeywordArg {
    keyword: Ident,
    value: Expr,
}

impl Parse for KeywordArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let keyword = input.parse()?;
        let _eq_token: Token![=] = input.parse()?;
        let value = input.parse()?;
        Ok(KeywordArg { keyword, value })
    }
}

#[derive(Debug)]
struct MacroConfig {
    output_name: Ident,
}

impl MacroConfig {
    fn new(input: &mut ItemMod) -> syn::Result<Self> {
        let target_enum = find_target_enum(input)?;
        Ok(MacroConfig {
            output_name: target_enum.ident.clone(),
        })
    }
}
