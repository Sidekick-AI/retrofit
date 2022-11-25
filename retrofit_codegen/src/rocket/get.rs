use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use rand::{distributions::Alphanumeric, prelude::StdRng, Rng, SeedableRng};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatIdent, PatType, Type};

pub fn get_api(header: TokenStream, function: TokenStream) -> TokenStream {
    let has_state = !header.to_string().replace(' ', "").is_empty();
    let input_fn = parse_macro_input!(function as ItemFn);

    // Get input function properties
    let mut args = input_fn.sig.inputs.clone();
    let (state, pass_through_state) = if has_state {
        let state = parse_macro_input!(header as Type);
        if args.len() < 2 {
            (quote! {state : &rocket::State<#state>}, quote! {&**state})
        } else {
            (
                quote! {, state : &rocket::State<#state>},
                quote! {, &**state},
            )
        }
    } else {
        (quote! {}, quote! {})
    };
    let return_type = input_fn.sig.output.clone();
    let input_fn_ident = input_fn.sig.ident.clone();

    // Create path for route
    let mut raw_args: Vec<&PatIdent> = input_fn
        .sig
        .inputs
        .iter()
        .map(|fn_arg| match fn_arg {
            FnArg::Typed(PatType { pat, .. }) => match &**pat {
                Pat::Ident(ident) => ident,
                _ => panic!("argument pattern is not a simple ident"),
            },
            FnArg::Receiver(_) => panic!("argument is a receiver"),
        })
        .collect();
    if has_state {
        // Remove last element because it is a state
        raw_args.pop();
        args.pop();
    }
    let arg_idents: Vec<Ident> = raw_args.iter().map(|i| i.ident.clone()).collect();
    // Ensure no args are vecs
    for arg in &args {
        assert!(
            !arg.into_token_stream()
                .to_string()
                .to_lowercase()
                .contains("vec"),
            "Vec arguments cannot be used in get apis. Try using a post api instead."
        )
    }

    let arg_types: Vec<(Ident, proc_macro2::TokenTree, bool)> = args
        .iter()
        .zip(arg_idents.iter())
        .map(|(i, ident)| {
            (
                ident.clone(),
                i.to_token_stream().into_iter().last().unwrap(),
                i.into_token_stream().to_string().contains('&'),
            )
        })
        .collect();
    let fn_input_args: Vec<proc_macro2::TokenStream> = arg_types
        .iter()
        .map(|(ident, _, r)| {
            if *r {
                quote! {&serde_json::from_str(&#ident).unwrap()}
            } else {
                quote! {serde_json::from_str(&#ident).unwrap()}
            }
        })
        .collect();
    let arg_ident_strings: Vec<String> = arg_idents.iter().map(|i| i.to_string()).collect();

    let route_path = if arg_ident_strings.is_empty() {
        format!("/{}", input_fn_ident)
    } else {
        format!(
            "/{}/{}",
            input_fn_ident,
            arg_ident_strings
                .iter()
                .map(|ident| format!("<{}>", ident))
                .collect::<Vec<String>>()
                .join("/")
        )
    };

    // Request path
    let input_fn_ident_string = input_fn_ident.to_string();
    let request_path_strings: Vec<proc_macro2::TokenStream> = raw_args
        .iter()
        .map(|arg| {
            let ident = arg.ident.clone();
            quote! {serde_json::to_string( & #ident.clone() ).unwrap()}
        })
        .collect();

    // Reqwest cannot take relative urls so we put localhost (the base url should be an option defined in the macro)
    let request_path = if request_path_strings.is_empty() {
        quote! {&format!("http://localhost:8000/{}", #input_fn_ident_string)}
    } else {
        quote! {&format!("http://localhost:8000/{}/{}", #input_fn_ident_string, [#(#request_path_strings),*].join("/"))}
    };
    // Reqwasm is able to take relative urls
    let reqwasm_request_path = if request_path_strings.is_empty() {
        quote! {&format!("/{}", #input_fn_ident_string)}
    } else {
        quote! {&format!("/{}/{}", #input_fn_ident_string, [#(#request_path_strings),*].join("/"))}
    };

    // Security type and random string
    let secure_struct_ident = Ident::new(
        &format!("{}Secure", input_fn_ident_string),
        Span::call_site(),
    );
    let mut s = DefaultHasher::new();
    input_fn_ident_string.hash(&mut s);
    let hash = s.finish();
    let secure_string = format!(
        "Bearer {}",
        StdRng::seed_from_u64(hash)
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>()
    );

    let route_ident = Ident::new(
        &format!("{}_route", input_fn_ident_string),
        Span::call_site(),
    );
    let request_ident = Ident::new(
        &format!("{}_request", input_fn_ident_string),
        Span::call_site(),
    );

    TokenStream::from(quote! {
        // Secure Struct
        #[derive(Debug)]
        pub struct #secure_struct_ident;

        #[cfg(feature = "server")]
        #[rocket::async_trait]
        impl<'r> rocket::request::FromRequest<'r> for #secure_struct_ident {
            type Error = String;

            async fn from_request(req: &'r rocket::Request<'_>) -> rocket::request::Outcome<Self, Self::Error> {
                match req.headers().get_one("authorization") {
                    Some(h) => {
                        if h == #secure_string { // Random string that will be in the header of every request
                            rocket::request::Outcome::Success(#secure_struct_ident)
                        } else {
                            rocket::request::Outcome::Forward(())
                        }
                    }
                    None => rocket::request::Outcome::Forward(()),
                }
            }
        }

        // Original function
        #[cfg(feature = "server")]
        #[allow(clippy::ptr_arg)]
        #input_fn

        // Route function
        #[cfg(feature = "server")]
        #[rocket::get(#route_path)]
        pub fn #route_ident ( #(#arg_idents : String),* #state, _secure: #secure_struct_ident) -> String {
            serde_json::to_string(& #input_fn_ident ( #(#fn_input_args),* #pass_through_state)).unwrap()
        }

        // Request function
        #[cfg(feature = "client")]
        #[allow(clippy::ptr_arg)]
        pub async fn #request_ident ( #args ) #return_type {
            // Send request to endpoint
            #[cfg(not(target_family = "wasm"))]
            return serde_json::from_str(
                &reqwest::Client::new()
                .get(#request_path)
                .header("authorization", #secure_string)
                .send().await.unwrap()
                .text().await.unwrap()
            ).unwrap();

            #[cfg(target_family = "wasm")]
            return reqwasm::http::Request::get(#reqwasm_request_path)
                .header("authorization", #secure_string)
                .send().await.unwrap()
                .json().await.unwrap();
        }
    })
}
