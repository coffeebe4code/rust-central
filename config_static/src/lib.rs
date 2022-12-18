use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{from_str, Value};
use syn::{parse_macro_input, DeriveInput, Type};

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
        syn::Data::Struct(ref data) => data
            .fields
            .iter()
            .map(|field| {
                let field_name = &field.ident.clone().unwrap();
                let u_json = json
                    .get(&field_name.to_string())
                    .expect(&*format!("missing json value {}", field_name.to_string()));
                // the ultimate intent is to verify that the config file provided is completely
                // serializable at compile time, and no fields in the Config Struct are missing
                // from the config file. (right now lazy static rereads the data, and deserializes
                // on startup to be lazy. later we can fill the object at compile time.
                match &field.ty {
                    // only path is hit
                    Type::Array(typed_arr) => {
                        println!("sized array");
                        if u_json.is_array() {
                        } else {
                            panic!("{:?} is not a correctly sized array", field_name);
                        }
                    }
                    // only path is hit
                    Type::Slice(typed_arr) => {
                        println!("dynamic array");
                        if !u_json.is_array() {
                            panic!(
                                "{:?} is not a correctly dynamically sized array",
                                field_name
                            );
                        }
                    }
                    // only this is hit, so we rely on the array being cast correctly for now.
                    Type::Path(path) => {
                        println!("path");
                        if path.path.is_ident("String") {
                            if !u_json.is_string() {
                                panic!("{:?} is not a string", field_name);
                            }
                        }
                    }

                    _ => {
                        panic!("{:?} is not a supported json castable item", field_name);
                    }
                }
            })
            .collect::<Vec<_>>(),
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
