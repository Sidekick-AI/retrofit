use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, Type, FnArg, PatType, Pat, PatIdent, Attribute, token::Ref};
use quote::{quote, ToTokens};
use proc_macro2::{Ident, Span};

pub fn post_api(header: TokenStream, function: TokenStream) -> TokenStream {
    let has_state = !header.to_string().replace(" ", "").is_empty();
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
    let tmp: Vec<(Ident, Type)> = raw_args.iter().map(|i| (i.ident.clone(), i.attrs[0].)).collect();
    println!("Args: {:?}", quote!{#args});

    let input_fn_ident_string = input_fn_ident.to_string();
    let route_ident = Ident::new(&format!("{}_route", input_fn_ident_string), Span::call_site());
    let request_ident = Ident::new(&format!("{}_request", input_fn_ident_string), Span::call_site());
    let data_struct_ident = Ident::new(&format!("{}Data", input_fn_ident_string), Span::call_site());

    let route_path = format!("/{}", input_fn_ident);
    let unpack_args = if args.is_empty() {quote!{}} else {quote!{let #data_struct_ident{ #(#arg_idents),* } = (*data).clone();}};
    let (route_header, route_args, pass_through_state) = if args.is_empty() {
        if has_state {
            let state = parse_macro_input!(header as Type);
            (quote!{#[rocket::post(#route_path)]}, quote!{state : &rocket::State<#state>}, quote!{&**state})
        } else {
            (quote!{#[rocket::post(#route_path)]}, quote!{}, quote!{})
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
        #input_fn

        // Data Struct
        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        #[allow(non_camel_case_types)]
        pub struct #data_struct_ident {
            #args
        }

        // Route function
        #[cfg(feature = "server")]
        #route_header
        pub fn #route_ident ( #route_args ) -> String {
            #unpack_args
            serde_json::to_string(
                & #input_fn_ident ( #(#arg_idents),* #pass_through_state)
            ).unwrap()
        }

        // Request function
        #[cfg(feature = "client")]
        pub async fn #request_ident ( #args ) #return_type {
            // Send request to endpoint
            #[cfg(not(target_family = "wasm"))]
            return serde_json::from_str(
                &reqwest::Client::new()
                .post(#request_path)
                #attached_body
                .send().await.unwrap()
                .text().await.unwrap()
            ).unwrap();

            #[cfg(target_family = "wasm")]
            return reqwasm::http::Request::post(#reqwasm_request_path)
                #attached_body
                .send().await.unwrap()
                .json().await.unwrap();
        }
    })
}