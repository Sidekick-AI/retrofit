mod api;
mod routes;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn api(header: TokenStream, function: TokenStream) -> TokenStream {
    api::api(header, function)
}

#[proc_macro_attribute]
pub fn routes_module(header: TokenStream, inner: TokenStream) -> TokenStream {
    routes::routes_module(header, inner)
}

#[proc_macro]
pub fn routes(inner: TokenStream) -> TokenStream {
    routes::routes(inner)
}
