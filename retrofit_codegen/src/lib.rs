mod get;
mod post;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn get_api(header: TokenStream, function: TokenStream) -> TokenStream {
    get::get_api(header, function)
}

#[proc_macro_attribute]
pub fn post_api(header: TokenStream, function: TokenStream) -> TokenStream {
    post::post_api(header, function)
}