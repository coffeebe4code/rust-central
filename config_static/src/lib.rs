use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{from_str, Value};
use syn::{parse_macro_input, DeriveInput};

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
                let field_name = &field.ident;
                let u_field = field_name.clone().unwrap().to_string();
                let u_json = json
                    .get(u_field.clone())
                    .expect(&*format!("missing json value {u_field}"))
                    .as_str()
                    .unwrap();
                let version = String::from(u_json);
                quote! {
                    #field_name: #version,
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
