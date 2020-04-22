use proc_macro2::TokenStream;
use syn::*;
use quote::quote;

#[proc_macro_attribute]
pub fn test_(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item_fn = parse_macro_input!(item as ItemFn);
    let block = &item_fn.block;
    let name = &item_fn.sig.ident;

    let code = quote! {
        test_wrapper::register_test! {
            test_wrapper::Test {
                name: std::stringify!(#name),
                file: std::file!(),
                line: std::line!(),
                handler: std::sync::Arc::new(
                    std::boxed::Box::new(|| {
                        #block
                    })
                ),
            }
        }
    };

    proc_macro::TokenStream::from(code)
}
