use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens, format_ident};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatIdent, PatType, Type, ReturnType};

pub fn api(header: TokenStream, function: TokenStream) -> TokenStream {
    let has_state = !header.to_string().replace(' ', "").is_empty();
    let input_fn = parse_macro_input!(function as ItemFn);

    // Get input function properties
    let mut args = input_fn.sig.inputs.clone();

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
    let arg_types: Vec<(Ident, proc_macro2::TokenStream, bool)> = args
        .iter()
        .zip(arg_idents.iter())
        .map(|(i, ident)| {
            (
                ident.clone(),
                if i.into_token_stream().to_string().contains('&') {
                    let string = i.to_token_stream().to_string();
                    let string = proc_macro2::TokenStream::from_str(
                        &string[string.find('&').unwrap() + 1..],
                    )
                    .unwrap();
                    quote! {#string}
                } else {
                    let string = i.to_token_stream().to_string();
                    let string = proc_macro2::TokenStream::from_str(
                        &string[string.find(':').unwrap() + 1..],
                    )
                    .unwrap();
                    quote! {#string}
                },
                i.into_token_stream().to_string().contains('&'),
            )
        })
        .collect();

    let fn_input_args: Vec<proc_macro2::TokenStream> = arg_types
        .iter()
        .map(|(ident, _, r)| {
            if *r {
                quote! {& #ident}
            } else {
                quote! {#ident}
            }
        })
        .collect();
    let struct_types: Vec<proc_macro2::TokenStream> = arg_types
        .iter()
        .map(|(ident, ty, _)| quote! {#ident: #ty})
        .collect();

    let input_fn_ident_string = input_fn.sig.ident.to_string();
    let route_ident = format_ident!("{}_route", input_fn_ident_string);
    let request_ident = format_ident!("{}_request", input_fn_ident_string);
    let data_struct_ident = format_ident!("{}Data", input_fn_ident_string);

    let (route_args, pass_through_state) = if args.is_empty() {
        if has_state {
            let state = parse_macro_input!(header as Type);
            (
                quote! {axum::extract::State(state) : axum::extract::State<#state>
                },
                quote! {&state},
            )
        } else {
            (
                quote! {},
                quote! {},
            )
        }
    } else if has_state {
        let state = parse_macro_input!(header as Type);
        (
            quote! {
                axum::extract::State(state) : axum::extract::State<#state>,
                axum::Json(#data_struct_ident{ #(#arg_idents),* }) : axum::Json<#data_struct_ident>,
            },
            quote! {, &state},
        )
    } else {
        (
            quote! {axum::Json(#data_struct_ident{ #(#arg_idents),* }) : axum::Json<#data_struct_ident>
            },
            quote! {},
        )
    };

    // Reqwest cannot take relative urls so we put localhost (the base url should be an option defined in the macro)
    let request_path = quote! {&format!("http://localhost:8000/{}", #input_fn_ident_string)};
    // Reqwasm is able to take relative urls
    let reqwasm_request_path = quote! {&format!("/{}", #input_fn_ident_string)};

    let attached_body = if args.is_empty() {
        quote! {}
    } else {
        quote! {
            .body(serde_json::to_string(
                &#data_struct_ident {
                    #(#arg_idents: #arg_idents.to_owned()),*
                }
            ).unwrap()).header("Content-Type", "application/json")
        }
    };

    let input_fn_ident = input_fn.sig.ident.clone();
    let return_type = match input_fn.sig.output {
        ReturnType::Default => quote!{()},
        ReturnType::Type(_, ref ty) => quote!{#ty},
    };
    TokenStream::from(quote! {
        // Original function
        #[cfg(feature = "server")]
        #[allow(clippy::ptr_arg)]
        #input_fn

        // Data Struct
        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        #[allow(non_camel_case_types)]
        pub struct #data_struct_ident {
            #(#struct_types),*
        }

        // Route function
        #[cfg(feature = "server")]
        async fn #route_ident ( #route_args ) -> String {
            serde_json::to_string(
                & #input_fn_ident ( #(#fn_input_args),* #pass_through_state)
            ).unwrap()
        }

        // Request function
        #[cfg(feature = "client")]
        #[allow(clippy::ptr_arg)]
        pub async fn #request_ident ( #args ) -> anyhow::Result<#return_type> {
            // Send request to endpoint
            #[cfg(not(target_family = "wasm"))]
            return Ok(reqwest::Client::new()
                .post(#request_path)
                #attached_body
                .send().await?
                .json().await?);

            #[cfg(target_family = "wasm")]
            return Ok(reqwasm::http::Request::post(#reqwasm_request_path)
                #attached_body
                .send().await?
                .json().await?);
        }
    })
}
