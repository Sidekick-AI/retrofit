#![feature(custom_inner_attributes)]
#![feature(extend_one)]

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
pub fn generate_rocket_routes(header: TokenStream, function: TokenStream) -> TokenStream {
    rocket::generate_rocket_routes(header, function)
}