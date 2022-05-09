# retrofit
A library for generating REST APIs for functions.

Currently supports generating `get` and `post` APIs.

When the `#[get_api]` or `#[post_api]` macro is used, a request function is generated using `reqwasm` if on wasm or `reqwest` otherwise, and a `rocket` route. 

## Usage
```rust
// Initial function
#[post_api]
fn plus(num1: i32, num2: i32) -> i32 {
    num1 + num2
}

// Normal usage
let result = plus(9, 10);

// Calling the function from client side
let result = plus_request(9, 10).await;

// Mounting the route on the server
#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![plus_route])
}

// The same works with GET APIs, but usually POST is recommended because it is more robust and can handle Vecs
#[get_api]
fn minus(num1: i32, num2: i32) -> i32 {
    num1 - num2
}

// State can also be used!
#[post_api(ConnectionPool)]
pub fn get_all_customers(pool: &ConnectionPool) -> Result<Vec<Customer>, Error> {
    Ok(customers::table.limit(i64::MAX)
        .load::<Customer>(&mut pool.get().unwrap())?)
}
// In this example, ConnectionPool is mounted on the rocket server, and does not need to be passed in to the request function
```

The example above generates a lot of code in the background. To get a better understanding of what Retrofit does, lets look at what's generated in the GET request minus function.

## Code Generated
```rust
// From above
#[get_api]
fn minus(num1: i32, num2: i32) -> i32 {
    num1 - num2
}

// Everything below this point is generated.

// Route generated
#[get("/minus/<num1>/<num2>")]
fn minus_route(num1: String, num2: String) -> String {
    serde_json::to_string(
        &minus(
            serde_json::from_str(&num1).unwrap(),
            serde_json::from_str(&num2).unwrap()
        )
    ).unwrap()
}

// Request function generated
async fn minus_request(num1: i32, num2: i32) -> i32 {
    serde_json::from_str(
        &reqwest::get(
            &format!("http://localhost:8000/{}/{}", 
                "minus", 
                serde_json::to_string(&num1).unwrap(), 
                serde_json::to_string(&num2).unwrap()
            )
        ).await.unwrap()
        .text().await.unwrap()
    ).unwrap()
}
```

## Feature Flags
`server` - contains generated Rocket routes

`client` - contains generated request function

These features serve as a way to use the same code on a server and client. By default they are both enabled, but if you have functions with a generated API in a crate of your own, and you use client/server feature flags, you can import the same crate on both backend and frontend, and your backend code won't be packaged with frontend code. This is usefull if you are targeting WASM on the frontend and using a DB system like Diesel (not supported on WASM) for your backend.

## Todo
- [x] Basic `get_api`
- [x] Add `server` and `client` feature support (allows the exclusion of the route/request functions)
- [ ] Support async functions
- [X] Add `post_api`
- [x] Support references
- [ ] Support generics
- [ ] Support other HTTP client libraries (surf, etc.)
- [ ] Support other HTTP server libraries (warp, actix, etc.)
- [X] Support Rocket States
- [ ] Better ergonomics for Rocket States (no attributes passed in, state not required to be last arg)
