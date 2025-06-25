use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident, LitInt};

extern crate proc_macro;

#[proc_macro]
pub fn integration_test(tokens: TokenStream) -> TokenStream {
    let integration_test_index = parse_macro_input!(tokens as LitInt);
    let input_mod_name = Ident::new(
        &format!("input_{}", integration_test_index),
        Span::call_site(),
    );
    let golden_mod_name = Ident::new(
        &format!("golden_{}", integration_test_index),
        Span::call_site(),
    );
    let test_function_name = Ident::new(
        &format!("integration_test_{}", integration_test_index),
        Span::call_site(),
    );
    let tokens = quote! {
        #[allow(dead_code)]
        mod #input_mod_name;
        mod #golden_mod_name;
        #[test]
        fn #test_function_name () {
            integration_test( #integration_test_index );
        }
    };
    tokens.into()
}
