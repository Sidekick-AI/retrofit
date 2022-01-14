use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, Type, FnArg, PatType, Pat, PatIdent};
use quote::{quote};
use proc_macro2::{Ident, Span};

pub fn get_api(header: TokenStream, function: TokenStream) -> TokenStream {
    let has_state = !header.to_string().replace(" ", "").is_empty();
    let input_fn = parse_macro_input!(function as ItemFn);
    
    // Get input function properties
    let mut args = input_fn.sig.inputs.clone();
    let (state, pass_through_state) = if has_state {
        let state = parse_macro_input!(header as Type);
        if args.len() < 2 {
            (quote! {state : &rocket::State<#state>}, quote!{&**state})
        } else {
            (quote! {, state : &rocket::State<#state>}, quote!{, &**state})
        }
    } else {(quote!{}, quote!{})};
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
    let arg_ident_strings: Vec<String> = arg_idents.iter().map(|i| i.to_string()).collect();

    let route_path = if arg_ident_strings.is_empty() {
        format!("/{}", input_fn_ident.to_string())
    } else {
        format!("/{}/{}", input_fn_ident.to_string(), arg_ident_strings.iter().map(|ident| format!("<{}>", ident)).collect::<Vec<String>>().join("/"))
    };
    // Request path
    let input_fn_ident_string = input_fn_ident.to_string();
    let request_path_strings: Vec<proc_macro2::TokenStream> = raw_args.iter().map(|arg| {
        let ident = arg.ident.clone();
        quote! {serde_json::to_string( & #ident.clone() ).unwrap()}
    }).collect();
    // Reqwest cannot take relative urls so we put localhost (the base url should be an option defined in the macro)
    let request_path = if request_path_strings.is_empty() {
        quote!{&format!("http://localhost:8000/{}", #input_fn_ident_string)} 
    } else {
        quote! {&format!("http://localhost:8000/{}/{}", #input_fn_ident_string, [#(#request_path_strings),*].join("/"))}
    };
    // Reqwasm is able to take relative urls
    let reqwasm_request_path = if request_path_strings.is_empty() {
        quote!{&format!("/{}", #input_fn_ident_string)}
    } else {
        quote!{&format!("/{}/{}", #input_fn_ident_string, [#(#request_path_strings),*].join("/"))}
    };

    let route_ident = Ident::new(&format!("{}_route", input_fn_ident_string), Span::call_site());
    let request_ident = Ident::new(&format!("{}_request", input_fn_ident_string), Span::call_site());

    TokenStream::from(quote!{
        // Original function
        #[cfg(feature = "server")]
        #input_fn

        // Route function
        #[cfg(feature = "server")]
        #[rocket::get(#route_path)]
        pub fn #route_ident ( #(#arg_idents : String),* #state) -> String {
            serde_json::to_string(& #input_fn_ident ( #(serde_json::from_str(&#raw_args).unwrap()),* #pass_through_state)).unwrap()
        }

        // Request function
        #[cfg(feature = "client")]
        pub async fn #request_ident ( #args ) #return_type {
            // Send request to endpoint
            #[cfg(not(target_family = "wasm"))]
            return serde_json::from_str(
                &reqwest::get(#request_path)
                .await.unwrap()
                .text().await.unwrap()
            ).unwrap();

            #[cfg(target_family = "wasm")]
            return reqwasm::http::Request::get(#reqwasm_request_path)
                .send().await.unwrap()
                .json().await.unwrap();
        }
    })
}