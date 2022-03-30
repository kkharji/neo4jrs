use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::ItemStruct;

fn main() {
    let s = "
    struct User { 
        pub username: String, 
        pub password: Option<String>, 
        pub created_at: NaiveDateTime
    }
    ";

    let tokens = TokenStream::from_str(s).unwrap();

    // build the AST: note the syn::parse2() method rather
    // than the syn::parse() one which is
    // meant for "real" procedural macros
    let ast: ItemStruct = syn::parse2(tokens).unwrap();

    // save our struct type for future use
    let struct_type = ast.ident.to_string();
    assert_eq!(struct_type, "User");

    // we have 3 fields
    assert_eq!(ast.fields.len(), 3);

    // syn::Fields is implementing the Iterator trait, so we can iterate through the fields
    let mut iter = ast.fields.iter();

    // this is username
    let username_field = iter.next().unwrap();
    assert_eq!(username_field.ident.as_ref().unwrap(), "username");

    // this is password
    let y_field = iter.next().unwrap();
    assert_eq!(y_field.ident.as_ref().unwrap(), "password");

    // this is created_at
    let password_field = iter.next().unwrap();
    assert_eq!(password_field.ident.as_ref().unwrap(), "created_at");

    // now the most tricky part: use the quote!() macro to generate code, aka a new
    // TokenStream

    // first, build our function name: user_summation
    let function_name = format_ident!("{}_username", struct_type.to_lowercase());

    // and our argument type. If we don't use the format ident macro, the function prototype
    // will be: pub fn point_summation (pt : "User")
    let argument_type = format_ident!("{}", struct_type);

    // same for x and y
    let username = format_ident!("{}", username_field.ident.as_ref().unwrap());
    let password = format_ident!("{}", password_field.ident.as_ref().unwrap());

    // the quote!() macro is returning a new TokenStream. This TokenStream is returned to
    // the compiler in a "real" procedural macro
    let username_fn = quote! {
        pub fn #function_name(pt: &#argument_type) -> String {
            pt.#username
        }

        pub fn has_password(pt: &#argument_type) -> bool {
            pt.#password.is_some()
        }
    };

    // output our function as Rust code
    println!("{:#?}", username_fn);
}
