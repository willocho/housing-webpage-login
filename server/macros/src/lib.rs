use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn redirect_to_login(_attr: TokenStream, item: TokenStream) -> TokenStream {
    return item;
}

