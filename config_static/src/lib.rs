use proc_macro2::TokenStream;
use quote::quote;
use serde_json::{from_str, Value};
use syn::{parse_macro_input, DeriveInput, Type};

#[proc_macro_attribute]
pub fn config_static_json(
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

    let assertions: Vec<TokenStream> = match input.data {
        // If the input is a struct, loop through its fields
        syn::Data::Struct(ref data) => data
            .fields
            .iter()
            .map(|field| {
                let field_name = &field.ident.clone().unwrap();
                let field_type = &field.ty;
                let u_json = json
                    .get(&field_name.to_string())
                    .expect(&*format!("missing json value {}", field_name.to_string()));
                if u_json.is_string() {
                    return quote! {
                        const fn assert_string<T: IsString>() {}
                        const _: () = {
                            assert_string::<#field_type>();
                        };
                    };
                } else if u_json.is_boolean() {
                    return quote! {
                        const fn assert_bool<T: IsBoolean>() {}
                        const _: () = {
                            assert_bool::<#field_type>();
                        };
                    };
                } else if u_json.is_i64() {
                    return quote! {
                        const fn assert_i64<T: IsInt64>() {}
                        const _: () = {
                            assert_i64::<#field_type>();
                        };
                    };
                } else {
                    panic!("not a supported json field {:?}", field_name);
                }
            })
            .collect::<Vec<_>>(),
        _ => panic!("This attribute can only be used on structs"),
    };

    let expanded = quote! {
        use serde_json::from_str;
        use once_cell::sync::Lazy;

        trait IsString {}
        trait IsInt64 {}
        trait IsBoolean {}
        impl IsString for String {}
        impl IsInt64 for i64 {}
        impl IsBoolean for bool {}

        static INSTANCE: Lazy<#name> = Lazy::new(|| {
            let file_contents = std::fs::read_to_string(#file_path).unwrap();
            let val: #name = from_str(&file_contents).unwrap();
            val
        });

        pub fn get_config() -> &'static #name {
            &INSTANCE
        }
        #input
        #(#assertions)*
    };

    proc_macro::TokenStream::from(expanded)
}
