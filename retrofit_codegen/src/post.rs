use std::{str::FromStr, collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};

use proc_macro::TokenStream;
use rand::{distributions::Alphanumeric, Rng, prelude::StdRng, SeedableRng};
use syn::{parse_macro_input, ItemFn, Type, FnArg, PatType, Pat, PatIdent};
use quote::{quote, ToTokens};
use proc_macro2::{Ident, Span};

pub fn post_api(header: TokenStream, function: TokenStream) -> TokenStream {
    let has_state = !header.to_string().replace(' ', "").is_empty();
    let input_fn = parse_macro_input!(function as ItemFn);
    
    // Get input function properties
    let mut args = input_fn.sig.inputs.clone();
    let return_type = input_fn.sig.output.clone();
    let input_fn_ident = input_fn.sig.ident.clone();

    // Create path for route
    let mut raw_args: Vec<&PatIdent> = input_fn.sig.inputs.iter().map(|fn_arg| match fn_arg {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(ident) => ident,
            _ => panic!("argument pattern is not a simple ident"),
        }
        FnArg::Receiver(_) => panic!("argument is a receiver"),
    }).collect();
    if has_state { // Remove last element because it is a state
        raw_args.pop();
        args.pop();
    }
    let arg_idents: Vec<Ident> = raw_args.iter().map(|i| i.ident.clone()).collect();
    let arg_types: Vec<(Ident, proc_macro2::TokenStream, bool)> = args.iter().zip(arg_idents.iter()).map(|(i, ident)| 
        (
            ident.clone(),
            if i.into_token_stream().to_string().contains('&') {
                let string = i.to_token_stream().to_string();
                let string = proc_macro2::TokenStream::from_str(&string[string.find('&').unwrap() + 1..]).unwrap();
                quote!{#string}
            } else {
                let string = i.to_token_stream().to_string();
                let string = proc_macro2::TokenStream::from_str(&string[string.find(':').unwrap() + 1..]).unwrap();
                quote!{#string}
            },
            i.into_token_stream().to_string().contains('&')
        )
    ).collect();

    let fn_input_args: Vec<proc_macro2::TokenStream> = arg_types.iter().map(|(ident, _, r)| {
        if *r {
            quote!{& #ident}
        } else {
            quote!{#ident}
        }
    }).collect();
    let struct_types: Vec<proc_macro2::TokenStream> = arg_types.iter().map(|(ident, ty, _)| quote!{#ident: #ty}).collect();

    let input_fn_ident_string = input_fn_ident.to_string();
    let route_ident = Ident::new(&format!("{}_route", input_fn_ident_string), Span::call_site());
    let request_ident = Ident::new(&format!("{}_request", input_fn_ident_string), Span::call_site());
    let data_struct_ident = Ident::new(&format!("{}Data", input_fn_ident_string), Span::call_site());
    
    // Security type and random string
    let secure_struct_ident = Ident::new(&format!("{}Secure", input_fn_ident_string), Span::call_site());
    let mut s = DefaultHasher::new();
    input_fn_ident_string.hash(&mut s);
    let hash = s.finish();
    let secure_string = format!("Bearer {}", StdRng::seed_from_u64(hash)
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect::<String>());


    let route_path = format!("/{}", input_fn_ident);
    let unpack_args = if args.is_empty() {quote!{}} else {quote!{let #data_struct_ident{ #(#arg_idents),* } = (*data).clone();}};
    let (route_header, route_args, pass_through_state) = if args.is_empty() {
        if has_state {
            let state = parse_macro_input!(header as Type);
            (quote!{#[rocket::post(#route_path)]}, quote!{state : &rocket::State<#state>, _: #secure_struct_ident}, quote!{&**state})
        } else {
            (quote!{#[rocket::post(#route_path)]}, quote!{_: #secure_struct_ident}, quote!{})
        }
    } else if has_state {
        let state = parse_macro_input!(header as Type);
        (
            quote!{#[rocket::post(#route_path, format="json", data="<data>")]}, 
            quote!{data: rocket::serde::json::Json<#data_struct_ident>, state : &rocket::State<#state>},
            quote!{, &**state}
        )
    } else {
        (
            quote!{#[rocket::post(#route_path, format="json", data="<data>")]},
            quote!{data: rocket::serde::json::Json<#data_struct_ident>},
            quote!{}
        )
    };

    // Reqwest cannot take relative urls so we put localhost (the base url should be an option defined in the macro)
    let request_path = quote!{&format!("http://localhost:8000/{}", #input_fn_ident_string)};
    // Reqwasm is able to take relative urls
    let reqwasm_request_path = quote!{&format!("/{}", #input_fn_ident_string)};

    let attached_body = if args.is_empty() {
        quote!{}
    } else {
        quote! {
            .body(serde_json::to_string(
                &#data_struct_ident {
                    #(#arg_idents: #arg_idents.to_owned()),*
                }
            ).unwrap()).header("Content-Type", "application/json")
        }
    };

    TokenStream::from(quote!{
        // Original function
        #[cfg(feature = "server")]
        #[allow(clippy::ptr_arg)]
        #input_fn

        // Secure Struct
        #[derive(Debug)]
        pub struct #secure_struct_ident;

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

        // Data Struct
        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        #[allow(non_camel_case_types)]
        pub struct #data_struct_ident {
            #(#struct_types),*
        }

        // Route function
        #[cfg(feature = "server")]
        #route_header
        pub fn #route_ident ( #route_args ) -> String {
            #unpack_args
            serde_json::to_string(
                & #input_fn_ident ( #(#fn_input_args),* #pass_through_state)
            ).unwrap()
        }

        // Request function
        #[cfg(feature = "client")]
        #[allow(clippy::ptr_arg)]
        pub async fn #request_ident ( #args ) #return_type {
            // Send request to endpoint
            #[cfg(not(target_family = "wasm"))]
            return serde_json::from_str(
                &reqwest::Client::new()
                .post(#request_path)
                .header("authorization", #secure_string)
                #attached_body
                .send().await.unwrap()
                .text().await.unwrap()
            ).unwrap();

            #[cfg(target_family = "wasm")]
            return reqwasm::http::Request::post(#reqwasm_request_path)
                .header("authorization", #secure_string)
                #attached_body
                .send().await.unwrap()
                .json().await.unwrap();
        }
    })
}