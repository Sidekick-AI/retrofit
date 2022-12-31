use proc_macro2::{Ident, Span, TokenStream, TokenTree};
use quote::quote;

pub fn routes_module(
    _header: proc_macro::TokenStream,
    stream: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let stream = proc_macro2::TokenStream::from(stream);
    // Get module name and inner stream
    let mut stream_iter = stream.into_iter();
    stream_iter.next();
    let module_name = match stream_iter.next().unwrap() {
        TokenTree::Ident(i) => Ident::new(&i.to_string(), Span::call_site()),
        _ => {
            panic!("Attribute must be on a module")
        }
    };
    let stream = match stream_iter.next().unwrap() {
        TokenTree::Group(group) => group.stream(),
        _ => {
            panic!("Attribute must be on a module")
        }
    };

    let (mut found_api_tag, mut after_function, mut state, mut temp_state) =
        (false, false, None, "".to_string());
    let route_names: Vec<TokenStream> = parse_stream(
        stream.clone(),
        &mut found_api_tag,
        &mut after_function,
        &mut state,
        &mut temp_state,
    )
    .into_iter()
    .map(|(name, route)| {
        let ident = Ident::new(&route, Span::call_site());
        let string = format!("/{}", name);
        quote! {#string, axum::routing::post(#ident)}
    })
    .collect();

    match state {
        Some(state) => {
            let state: proc_macro2::TokenStream = state.parse().unwrap();
            proc_macro::TokenStream::from(quote! {
                mod #module_name {
                #[cfg(feature = "server")]
                pub fn routes() -> axum::Router<#state> {
                    axum::Router::new()
                        #( .route(#route_names) )*
                }

                #stream
                }
            })
        }
        None => proc_macro::TokenStream::from(quote! {
            mod #module_name {
            #[cfg(feature = "server")]
            pub fn routes() -> axum::Router {
                axum::Router::new()
                    #( .route(#route_names) )*
            }

            #stream
            }
        }),
    }
}

pub fn routes(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = proc_macro2::TokenStream::from(stream);

    let (mut found_api_tag, mut after_function, mut state, mut temp_state) =
        (false, false, None, "".to_string());
    let route_names: Vec<TokenStream> = parse_stream(
        stream.clone(),
        &mut found_api_tag,
        &mut after_function,
        &mut state,
        &mut temp_state,
    )
    .into_iter()
    .map(|(name, route)| {
        let ident = Ident::new(&route, Span::call_site());
        let string = format!("/{}", name);
        quote! {#string, axum::routing::post(#ident)}
    })
    .collect();

    match state {
        Some(state) => {
            let state: proc_macro2::TokenStream = state.parse().unwrap();
            proc_macro::TokenStream::from(quote! {
                #[cfg(feature = "server")]
                pub fn routes() -> axum::Router<#state> {
                    axum::Router::new()
                        #( .route(#route_names) )*
                }

                #stream
            })
        }
        None => proc_macro::TokenStream::from(quote! {
            #[cfg(feature = "server")]
            pub fn routes() -> axum::Router {
                axum::Router::new()
                    #( .route(#route_names) )*
            }

            #stream
        }),
    }
}

/// Parse a TokenStream into a vec of route names
fn parse_stream(
    stream: TokenStream,
    found_api_tag: &mut bool,
    after_function: &mut bool,
    state: &mut Option<String>,
    temp_state: &mut String,
) -> Vec<(String, String)> {
    let mut route_names = vec![];
    for tree in stream.into_iter() {
        match tree {
            TokenTree::Ident(i) => {
                let string = i.to_string();
                if *after_function {
                    if *found_api_tag {
                        route_names.push((
                            string.clone(),
                            format!("{}_route", string),
                        ));
                        *found_api_tag = false;
                    }
                    *after_function = false;
                }
                if string == "fn" {
                    *after_function = true;

                    if !temp_state.replace(' ', "").is_empty() {
                        match state {
                            Some(s) => {
                                if *s != temp_state.replace(' ', "") {
                                    panic!(
                                        "Only one type is allowed. First type: {} Second Type: {}",
                                        s,
                                        temp_state.replace(' ', "")
                                    );
                                }
                            }
                            None => {
                                *state = Some(temp_state.replace(' ', ""));
                                *temp_state = "".to_string();
                            }
                        }
                    }
                }

                if *found_api_tag && !*after_function && string != "pub" && string != "async" {
                    *temp_state = format!("{}{}", temp_state, string);
                }
                
                if string == "api" {
                    *found_api_tag = true;
                    *temp_state = "".to_string();
                }
            }
            TokenTree::Group(group) => {
                route_names.extend(
                    parse_stream(
                        group.stream(),
                        found_api_tag,
                        after_function,
                        state,
                        temp_state,
                    )
                    .into_iter(),
                );
            }
            TokenTree::Punct(p) => {
                if *found_api_tag && !*after_function {
                    *temp_state = format!("{}{}", temp_state, p);
                }
            }
            _ => {}
        }
    }

    route_names
}
