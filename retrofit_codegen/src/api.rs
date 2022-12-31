use std::str::FromStr;

use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, PatType, ReturnType, Type};

pub fn api(header: TokenStream, function: TokenStream) -> TokenStream {
    let has_state = !header.to_string().replace(' ', "").is_empty();
    let input_fn = parse_macro_input!(function as ItemFn);

    let mut args = input_fn.sig.inputs.clone();
    if has_state {
        // Remove last element because it is a state
        args.pop();
    }

    let arg_idents: Vec<_> = args
        .iter()
        .map(|fn_arg| match fn_arg {
            FnArg::Typed(PatType { pat, .. }) => match &**pat {
                Pat::Ident(ident) => ident.ident.clone(),
                _ => panic!("argument pattern is not a simple ident"),
            },
            FnArg::Receiver(_) => panic!("argument is a receiver"),
        })
        .collect();
    let (fn_input_args, struct_types): (Vec<_>, Vec<_>) = args
        .iter()
        .zip(arg_idents.iter())
        .map(|(i, ident)| {
            let s = i.into_token_stream().to_string();
            let ty = proc_macro2::TokenStream::from_str(if let Some(i) = s.find('&') {
                &s[i + 1..]
            } else {
                &s[s.find(':').unwrap() + 1..]
            })
            .unwrap();
            (
                if s.contains('&') {
                    quote! {& #ident}
                } else {
                    quote! {#ident}
                },
                quote! {#ident: #ty},
            )
        })
        .unzip();

    let input_fn_ident_string = input_fn.sig.ident.to_string();
    let data_struct_ident = format_ident!("{}Data", input_fn_ident_string);

    let route_args = if has_state {
        let state = parse_macro_input!(header as Type);
        quote! {
            axum::extract::State(state) : axum::extract::State<#state>,
            axum::Json(#data_struct_ident{ #(#arg_idents),* }) : axum::Json<#data_struct_ident>,
        }
    } else {
        quote! {axum::Json(#data_struct_ident{ #(#arg_idents),* }) : axum::Json<#data_struct_ident>}
    };
    let pass_through_state = if has_state {
        quote! {, &state}
    } else {
        quote! {}
    };

    let input_fn_ident = input_fn.sig.ident.clone();
    let return_type = match input_fn.sig.output {
        ReturnType::Default => quote! {()},
        ReturnType::Type(_, ref ty) => quote! {#ty},
    };
    let route_ident = format_ident!("{}_route", input_fn_ident_string);
    let request_ident = format_ident!("{}_request", input_fn_ident_string);
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
                .post(&format!("http://localhost:8000/{}", #input_fn_ident_string))
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(
                    &#data_struct_ident {
                        #(#arg_idents: #arg_idents.to_owned()),*
                    }
                ).unwrap())
                .send().await?
                .json().await?);

            #[cfg(target_family = "wasm")]
            return Ok(reqwasm::http::Request::post(&format!("/{}", #input_fn_ident_string))
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(
                    &#data_struct_ident {
                        #(#arg_idents: #arg_idents.to_owned()),*
                    }
                ).unwrap())
                .send().await?
                .json().await?);
        }
    })
}
