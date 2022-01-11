# retrofit
A library for generating REST APIs for functions.

Currently only has get_api, an implementation for generating get references.

When the #[get_api] macro is used, a request function is generated using `reqwasm` if on wasm or `reqwest` otherwise, and a `rocket` route. 

## Usage
```rust
// Initial function
#[get_api]
fn plus(num1: i32, num2: i32) -> i32 {
    num1 + num2
}

// Normal usage
let result = plus(9, 10);

// Route generated
#[get("/plus/<num1>/<num2>")]
fn plus_route(num1: String, num2: String) -> String {
    serde_json::to_string(
        &plus(
            serde_json::from_str(&num1).unwrap(),
            serde_json::from_str(&num2).unwrap()
        )
    ).unwrap()
}

// Request function generated
async fn plus_request(num1: i32, num2: i32) -> i32 {
    serde_json::from_str(
        &reqwest::get(
            &format!("http://localhost:8000/{}/{}", 
                "plus", 
                serde_json::to_string(&num1).unwrap(), 
                serde_json::to_string(&num2).unwrap()
            )
        ).await.unwrap()
        .text().await.unwrap()
    ).unwrap()
}
```

## Todo
- [x] Basic `get_api`
- [ ] Support async functions
- [ ] Add `post_api`
- [ ] Support references
- [ ] Support generics
