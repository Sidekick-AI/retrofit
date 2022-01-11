use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, FnArg, PatType, Pat};
use quote::{quote};
use proc_macro2::{Ident, Span};

#[proc_macro_attribute]
pub fn get_api(_: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    
    // Get input function properties
    let args = input_fn.sig.inputs.clone();
    let return_type = input_fn.sig.output.clone();
    let input_fn_ident = input_fn.sig.ident.clone();

    // Create path for route
    let raw_args = args.iter().map(|fn_arg| match fn_arg {
        FnArg::Typed(PatType { pat, .. }) => match &**pat {
            Pat::Ident(ident) => ident,
            _ => panic!("argument pattern is not a simple ident"),
        }
        FnArg::Receiver(_) => panic!("argument is a receiver"),
    });
    let arg_idents: Vec<Ident> = raw_args.clone().map(|i| i.ident.clone()).collect();
    let arg_ident_strings: Vec<String> = arg_idents.iter().map(|i| i.to_string()).collect();

    let route_path = if arg_ident_strings.is_empty() {
        format!("/{}", input_fn_ident.to_string())
    } else {
        format!("/{}/{}", input_fn_ident.to_string(), arg_ident_strings.iter().map(|ident| format!("<{}>", ident)).collect::<Vec<String>>().join("/"))
    };
    // Request path
    let input_fn_ident_string = input_fn_ident.to_string();
    let request_path_strings: Vec<proc_macro2::TokenStream> = raw_args.clone().map(|arg| {
        let ident = arg.ident.clone();
        quote! {serde_json::to_string( & #ident ).unwrap()}
    }).collect();
    let request_path = if request_path_strings.is_empty() {
        quote! {#input_fn_ident_string}
    } else {
        quote! {&format!("http://localhost:8000/{}/{}", #input_fn_ident_string, [#(#request_path_strings),*].join("/"))}
    };

    let route_ident = Ident::new(&format!("{}_route", input_fn_ident_string), Span::call_site());
    let request_ident = Ident::new(&format!("{}_request", input_fn_ident_string), Span::call_site());

    TokenStream::from(quote!{
        // Original function
        #input_fn
        
        // Route function
        #[rocket::get(#route_path)]
        pub fn #route_ident ( #(#arg_idents : String),* ) -> String {
            serde_json::to_string(& #input_fn_ident ( #(serde_json::from_str(&#raw_args).unwrap()),* )).unwrap()
        }

        // Request function
        pub async fn #request_ident ( #args ) #return_type {
            // Send request to endpoint
            #[cfg(not(target_arch = "wasm32"))]
            return serde_json::from_str(
                &reqwest::get(#request_path)
                .await.unwrap()
                .text().await.unwrap()
            ).unwrap();

            #[cfg(target_arch = "wasm32")]
            return reqwasm::http::Request::get(#request_path)
                .send().await.unwrap()
                .json().await.unwrap();
        }
    })
}