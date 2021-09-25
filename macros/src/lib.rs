extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod command;

#[proc_macro_derive(CommandOption, attributes(description, command_enum))]
pub fn command_handler_macro(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    command::derive_command_handler(ast).into()
}