extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use proc_macro2::Span;
use syn::*;
use heck::SnakeCase;

#[derive(Debug)]
struct ParsedCommandEnum {
    name: Ident,
    value: String,
    description: String
}

#[derive(Debug)]
enum ParsedCommandExtras {
    None, // No extra fields
    Choices(Ident), // Choice field, type implements CommandOption
    Options(Ident), // Options field, type implements CommandOption
    OptionsReal(Vec<ParsedCommandOption>) // Options field, we can construct the options
}

#[derive(Debug,PartialEq,Copy,Clone)]
enum ParsedCommandType {
    SubCommand      = 1,
    SubCommandGroup = 2,
    String          = 3,
    Integer         = 4,
    Boolean         = 5,
    User            = 6,
    Channel         = 7,
    Role            = 8,
    Mentionable     = 9,
    Number          = 10
}

fn code_for_ident(ident: &Ident) -> ParsedCommandType {
    use ParsedCommandType::*;
    let ident_str: &str = &ident.to_string();
    match ident_str {
        "String"      => String,
        "u64" | "i64" => Integer,
        "bool"        => Boolean,
        "User"        => User,
        "Channel"     => Channel,
        "Role"        => Role,
        "Mentionable" => Mentionable,
        "f64"         => Number,
        _             => SubCommandGroup
    }
}

#[derive(Debug)]
struct ParsedCommandOption {
    t_code: ParsedCommandType,
    t: Ident,
    name: Ident,
    description: String,
    required: bool,
    options: ParsedCommandExtras
}

impl ParsedCommandOption {
    fn to_json_quote(&self) -> proc_macro2::TokenStream {
        let option_quote = match &self.options {
            ParsedCommandExtras::None => quote!(),
            ParsedCommandExtras::Options(options) => quote!("options": #options::to_value()),
            ParsedCommandExtras::Choices(choices) => quote!("choices": #choices::to_value()),
            ParsedCommandExtras::OptionsReal(options) => {
                let options_quotes = options.iter().map(|option| option.to_json_quote());
                quote!("options": [#(#options_quotes),*])
            }
        };
        let name = self.name.to_string().to_snake_case();
        let ty = self.t_code as u8;
        let desc = self.description.to_owned();
        let required = self.required;
        quote!(
            {
                "name": #name,
                "type": #ty,
                "description": #desc,
                "required": #required,
                #option_quote
            }
        )
    }

    fn from_json_quote(&self, name: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        let idx = &self.name.to_string().to_snake_case();
        if let ParsedCommandExtras::OptionsReal(options) = &self.options {
            let param_quotes = options.iter().map(|option| {
                let name_str = &option.name.to_string();
                let name = &option.name;
                let cast = cast_quote(&option.t, quote!(args.get(#name_str)), option.required);
                quote!(#name: #cast)
            });
            let name = &self.name;
            let t = &self.t;
            quote!{
                #idx => {
                    Some( #t::#name {
                            #(#param_quotes),*
                        })
                }
            }
         } else {
            cast_quote(&self.t, quote!(#name[idx]), self.required)
        }
    }
}

fn find_desc(attrs: &Vec<Attribute>, default: String) -> String {
    attrs.to_owned().iter().find(|attr| {
        let ident = attr.path.get_ident();
        match ident {
            Some(ident) => ident.to_string() == "description",
            None => false
    }}).map_or(default.to_owned(), | attr | {
        let mut tokens = attr.tokens.to_owned().into_iter();
        match tokens.next() {
            Some(proc_macro2::TokenTree::Punct(next)) if next.as_char() == '=' => {
                if let Some(token) = tokens.next() {
                    let name = token.to_string();
                    String::from(name.split_at(name.len() - 1).0.split_at(1).1)
                } else {
                    default.to_owned()
                }
            }
            _ => default.to_owned()
        }
    })
}

fn attr_is_enum(attrs: &Vec<Attribute>) -> bool {
    attrs.iter().any(|attr| {
        match attr.path.get_ident() {
            Some(ident) => ident.to_string() == "command_enum",
            None => false
        }
    })
}

fn cast_quote(t: &Ident, value_name: proc_macro2::TokenStream, required: bool) -> proc_macro2::TokenStream {
    let type_name: &str = &t.to_string();
    let q = match type_name {
        "String" => quote!(#value_name.map(|val| val.as_str()).flatten().map(|val| val.to_string())),
        "bool"   => quote!(#value_name.map(|val| val.as_bool()).flatten()),
        "f64"    => quote!(#value_name.map(|val| val.as_f64()).flatten()),
        "u64"    => quote!(#value_name.map(|val| val.as_u64()).flatten()),
        "i64"    => quote!(#value_name.map(|val| val.as_i64()).flatten()),
        _ => quote!(#value_name.map(|val| #t::from_value(val)).flatten())
    };
    if required {
        quote!(#q?)
    } else {
        q
    }
}

use std::iter::repeat;
#[proc_macro_derive(CommandOption, attributes(description,command_enum))]
pub fn command_handler_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let name_r = repeat(name);

    let is_enum = attr_is_enum(&ast.attrs);

    let variants = match ast.data {
        Data::Enum(DataEnum{variants,..}) => variants, 
        _ => return Error::new(
            Span::mixed_site(),
            "Can only apply CommandOption to enums").to_compile_error().into()
    };
    let gen = if is_enum {
        let parsed_enum = variants.iter().map(|variant| {
            ParsedCommandEnum{name: variant.ident.to_owned(),
                value: variant.ident.to_string().to_snake_case(),
                description: find_desc(&variant.attrs, variant.ident.to_string())
            }
        });
        let ident_stream       = parsed_enum.to_owned().map(|item| item.name);
        let value_stream       = parsed_enum.to_owned().map(|item| item.value);
        let value_stream2      = value_stream.to_owned();
        let description_stream = parsed_enum.to_owned().map(|item| item.description);
        quote!{
            impl CommandOption for #name {
                fn from_value(option: &serde_json::Value) -> Option<Self>{
                    if let Some(o) = option.as_str() {
                        match o {
                            #( #value_stream => Some(#name_r::#ident_stream), )*
                            _ => None
                        }
                    } else {
                        None
                    }
                }

                #[cfg(any(feature = "keep_json", not(target_arch = "wasm32")))]
                fn to_value() -> serde_json::Value {
                    json!([#({"name": #description_stream, "value": #value_stream2}),*])
                }
            }
        }
    }
    else {
        let parsed_option = variants.iter().map(|variant| {
            let desc = find_desc(&variant.attrs, "".to_string());
            let args = variant.fields.iter().map(|field| {
                if let Type::Path(path) = &field.ty {
                    let arg = &path.path.segments[path.path.segments.len() - 1];
                    let desc = find_desc(&field.attrs, "".to_string());
                    let (ty, required) = match &arg.arguments {
                        PathArguments::None => (arg.ident.to_owned(), true),
                        PathArguments::AngleBracketed(args) => {
                            if let Some(GenericArgument::Type(Type::Path(path))) = args.args.last() {
                                let ty = &path.path.segments[path.path.segments.len() - 1];
                                let gen_ty: &str = &arg.ident.to_string();
                                match gen_ty {
                                    "Option" => (ty.ident.to_owned(), false),
                                    _        => panic!("Unreachable")
                                }
                            }
                            else {
                                panic!("Unreachable")
                            }
                        }
                        _ => panic!("Unreachable"),
                    };
                    let is_enum = attr_is_enum(&field.attrs);
                    let (code, ty, options) = if is_enum {
                        (ParsedCommandType::String, ty.to_owned(), ParsedCommandExtras::Choices(ty.to_owned()))
                    } else {
                        let code = code_for_ident(&ty);
                        let extras = if code == ParsedCommandType::SubCommandGroup {
                            ParsedCommandExtras::Options(ty.to_owned())
                        } else {
                            ParsedCommandExtras::None
                        };
                        (code, ty.to_owned(), extras)
                    };
                    ParsedCommandOption{
                        t_code: code,
                        t: ty,
                        name: field.ident.as_ref().expect("Only named fields (struct style enums) supported").to_owned(),
                        description: desc,
                        required: required,
                        options
                    }
                } else {
                    panic!("Error");
                }
            }).collect();
            ParsedCommandOption{
                t_code: ParsedCommandType::SubCommand,
                t: name.to_owned(),
                name: variant.ident.to_owned(),
                description: desc,
                required: true,
                options: ParsedCommandExtras::OptionsReal(args)}
        });
        let to_value_quotes = parsed_option.to_owned().map(|option| option.to_json_quote());
        let from_json_quotes = parsed_option.to_owned().map(|option| option.from_json_quote(quote!(options)));
        quote!{
            impl CommandOption for #name {
                fn from_value(options: &serde_json::Value) -> Option<Self>{
                    let args = std::collections::HashMap::<&str, &serde_json::Value, std::collections::hash_map::RandomState>::from_iter(
                        options.get("options")?
                        .as_array()?
                        .iter().map(|option| (option["name"].as_str().unwrap(), &option["value"])));

                    match options["name"].as_str().unwrap() {
                        #(#from_json_quotes),*
                        _ => None
                    }
                }

                #[cfg(any(feature = "keep_json", not(target_arch = "wasm32")))]
                fn to_value() -> serde_json::Value {
                    json!([
                        #(#to_value_quotes),*
                    ])
                }
            }
        }
    };
    gen.into()
}