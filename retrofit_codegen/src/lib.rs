mod axum;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn get_api(header: TokenStream, function: TokenStream) -> TokenStream {
    axum::get::get_api(header, function)
}

#[proc_macro_attribute]
pub fn post_api(header: TokenStream, function: TokenStream) -> TokenStream {
    axum::post::post_api(header, function)
}

#[proc_macro_attribute]
pub fn routes_module(header: TokenStream, inner: TokenStream) -> TokenStream {
    axum::routes::routes_module(header, inner)
}

#[proc_macro]
pub fn routes(inner: TokenStream) -> TokenStream {
    axum::routes::routes(inner)
}
