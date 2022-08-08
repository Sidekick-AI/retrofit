mod get;
mod post;
mod rocket;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn get_api(header: TokenStream, function: TokenStream) -> TokenStream {
    get::get_api(header, function)
}

#[proc_macro_attribute]
pub fn post_api(header: TokenStream, function: TokenStream) -> TokenStream {
    post::post_api(header, function)
}

#[proc_macro_attribute]
pub fn rocket_routes_module(header: TokenStream, inner: TokenStream) -> TokenStream {
    rocket::rocket_routes_module(header, inner)
}

#[proc_macro]
pub fn rocket_routes(inner: TokenStream) -> TokenStream {
    rocket::rocket_routes(inner)
}