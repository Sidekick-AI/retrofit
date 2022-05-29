use proc_macro2::{TokenStream, TokenTree, Ident, Span};
use quote::quote;

pub fn generate_rocket_routes(_header: proc_macro::TokenStream, stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream = proc_macro2::TokenStream::from(stream);
    // Get module name and inner stream
    let mut stream_iter = stream.into_iter();
    stream_iter.next();
    let module_name = match stream_iter.next().unwrap() {
        TokenTree::Ident(i) => Ident::new(&i.to_string(), Span::call_site()),
        _ => {panic!("Attribute must be on a module")}
    };
    let stream = match stream_iter.next().unwrap() {
        TokenTree::Group(group) => group.stream(),
        _ => {panic!("Attribute must be on a module")}
    };

    /// Parse a TokenStream into a vec of route names
    fn parse_stream(stream: TokenStream, found_api_tag: &mut bool, after_function: &mut bool) -> Vec<String> {
        let mut route_names = vec![];
        for tree in stream.into_iter() {
            match tree {
                TokenTree::Ident(i) => {
                    let string = i.to_string();
                    println!("{}", string);
                    if *after_function {
                        if *found_api_tag {
                            route_names.push(format!("{}_route", string));
                            *found_api_tag = false;
                        }
                        *after_function = false;
                    }
                    if string == "fn" {
                        *after_function = true;
                    }
                    if string == "get_api" || string == "post_api" {
                        *found_api_tag = true;
                    }
                },
                TokenTree::Group(group) => {
                    println!("Group");
                    route_names.extend(parse_stream(group.stream(), found_api_tag, after_function).into_iter());
                },
                _ => {}
            }
        }

        route_names
    }

    let (mut found_api_tag, mut after_function) = (false, false);
    let route_names: Vec<Ident> = parse_stream(stream.clone(), &mut found_api_tag, &mut after_function)
        .into_iter().map(|i| Ident::new(&i, Span::call_site())).collect();

    proc_macro::TokenStream::from(quote!{
        mod #module_name {
        pub fn rocket_routes() -> Vec<rocket::Route> {
            let routes = rocket::routes![
                #(#route_names),*
            ];
            routes
        }

        #stream
        }
    })
}