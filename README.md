# retrofit
A library for generating REST APIs for functions.

Currently only has get_api, an implementation for generating get references.

When the #[get_api] macro is used, a request function is generated using `reqwasm` if on wasm or `reqwest` otherwise, and a `rocket` route. 

## Usage
```
// Initial function
#[get_api]
fn plus(num1: i32, num2: i32) -> i32 {
    num1 + num2
}

// Normal usage
let result = plus(9, 10);

// 
```