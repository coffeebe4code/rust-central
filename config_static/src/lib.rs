use std::any::Any;

use quote::quote;
use serde_json::{from_str, Value};
use syn::{parse_macro_input, DeriveInput, Path, Type};

#[proc_macro_attribute]
pub fn config_static_check(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // Parse the file path from the attribute
    let file_path = attr.to_string();
    let file_path = file_path.trim().trim_matches('"');

    // Read the file and parse it as a JSON object
    let file_contents = std::fs::read_to_string(file_path).unwrap();
    let json: Value = from_str(&file_contents).unwrap();

    // Parse the input struct
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    match input.data {
        // If the input is a struct, loop through its fields
        syn::Data::Struct(ref data) => data.fields.iter().for_each(|field| {
            let field_name = &field.ident.clone().unwrap();
            let u_json = json
                .get(&field_name.to_string())
                .expect(&*format!("missing json value {}", field_name.to_string()));
            recurse_path(&field.ty, &u_json, &field_name.to_string());
        }),
        _ => panic!("This attribute can only be used on structs"),
    };

    let expanded = quote! {
        use serde_json::from_str;
        use once_cell::sync::Lazy;

        static INSTANCE: Lazy<#name> = Lazy::new(|| {
            let file_contents = std::fs::read_to_string(#file_path).unwrap();
            let val: #name = from_str(&file_contents).unwrap();
            val
        });

        pub fn get_config() -> &'static #name {
            &INSTANCE
        }
        #input
    };

    proc_macro::TokenStream::from(expanded)
}

fn recurse_path(the_ty: &Type, json: &Value, field_name: &String) -> () {
    match the_ty {
        Type::Path(path) => {
            if json.is_string() {
                assert!(path.path.is_ident("String"));
            } else if json.is_i64() {
                assert!(path.path.is_ident("i64"));
            } else if json.is_array() {
                path.path.segments.iter().for_each(|s| {
                    println!("s {}", s.ident.to_string());
                    match &s.arguments {
                        syn::PathArguments::AngleBracketed(arg) => {
                            print!("{:?}", arg.type_id());
                        }
                        _ => {
                            panic!(
                                "not a valid path argument at {field_name}, with {:?}",
                                s.ident.to_string()
                            );
                        }
                    }
                });
            }
        }
        _ => {
            panic!("{:?} is not a supported json castable item", field_name);
        }
    }
}
