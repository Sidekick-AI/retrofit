#[cfg(feature = "axum")]
mod axum;
#[cfg(feature = "rocket")]
mod rocket;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn get_api(header: TokenStream, function: TokenStream) -> TokenStream {
    cfg_if::cfg_if! {
        if #[cfg(feature="rocket")] {
            rocket::get::get_api(header, function)
        } else if #[cfg(feature="axum")] {
            axum::get::get_api(header, function)
        }
    }
}

#[proc_macro_attribute]
pub fn post_api(header: TokenStream, function: TokenStream) -> TokenStream {
    cfg_if::cfg_if! {
        if #[cfg(feature="rocket")] {
            rocket::post::post_api(header, function)
        } else if #[cfg(feature="axum")] {
            axum::post::post_api(header, function)
        }
    }
}

#[proc_macro_attribute]
pub fn routes_module(header: TokenStream, inner: TokenStream) -> TokenStream {
    cfg_if::cfg_if! {
        if #[cfg(feature="rocket")] {
            rocket::routes::routes_module(header, inner)
        } else if #[cfg(feature="axum")] {
            axum::routes::routes_module(header, inner)
        }
    }
}

#[proc_macro]
pub fn routes(inner: TokenStream) -> TokenStream {
    cfg_if::cfg_if! {
        if #[cfg(feature="rocket")] {
            rocket::routes::routes(inner)
        } else if #[cfg(feature="axum")] {
            axum::routes::routes(inner)
        }
    }
}
