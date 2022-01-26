pub use reqwasm;
pub use reqwest;
use retrofit_codegen::{get_api, post_api};
pub use rocket;
use rocket::routes;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;

#[tokio::test]
#[serial_test::serial]
async fn test_get_api() {
    // Test a normal GET API
    #[get_api]
    fn plus(num1: i32, num2: i32) -> i32 {
        num1 + num2
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![plus_route])
            .launch()
            .await
    });

    let input1 = 10;
    let input2 = 100;
    // Call request
    let result = plus_request(input1, input2).await;
    assert_eq!(result, plus(input1, input2));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
#[serial_test::serial]
async fn test_get_api_state() {
    // Test a GET API with a managed state
    #[get_api(Mutex<String>)]
    fn greet(name: String, state: &Mutex<String>) -> String {
        let mut state = state.lock().unwrap();
        let greeting = format!("Hello {}, I'm here with {}", name, state);
        *state = name;
        greeting
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![greet_route])
            .manage(Mutex::new("Robert".to_string()))
            .launch()
            .await
    });

    assert_eq!(
        greet_request("Joe".to_string()).await,
        "Hello Joe, I'm here with Robert".to_string()
    );
    assert_eq!(
        greet_request("Frank".to_string()).await,
        "Hello Frank, I'm here with Joe".to_string()
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
#[serial_test::serial]
async fn test_post_api() {
    #[post_api]
    fn plus(num1: i32, num2: i32) -> i32 {
        num1 + num2
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![plus_route])
            .launch()
            .await
    });

    let input1 = 10;
    let input2 = 100;
    // Call request
    let result = plus_request(input1, input2).await;
    assert_eq!(result, plus(input1, input2));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
#[serial_test::serial]
async fn test_post_api_state() {
    // Test a GET API with a managed state
    #[post_api(Mutex<String>)]
    fn greet(name: String, state: &Mutex<String>) -> String {
        let mut state = state.lock().unwrap();
        let greeting = format!("Hello {}, I'm here with {}", name, state);
        *state = name;
        greeting
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![greet_route])
            .manage(Mutex::new("Robert".to_string()))
            .launch()
            .await
    });

    assert_eq!(
        greet_request("Joe".to_string()).await,
        "Hello Joe, I'm here with Robert".to_string()
    );
    assert_eq!(
        greet_request("Frank".to_string()).await,
        "Hello Frank, I'm here with Joe".to_string()
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
#[serial_test::serial]
async fn test_post_api_ref() {
    // Test POST API with references
    #[post_api]
    fn greet<'a>(nm: &'a str) -> String {
        format!("Hello {}", nm)
    }

    // Should generate
    // fn greet(name: &str) -> String {
    //     format!("Hello {}", name)
    // }

    // #[derive(Serialize, Deserialize, Clone)]
    // pub struct greetData {
    //     name: String
    // }

    // #[rocket::post("/greet_route", format="json", data="<data>")]
    // fn greet_route(data: rocket::serde::json::Json<greetData>) -> String {
    //     let greetData{name} = (*data).clone();
    //     greet(&name)
    // }

    // async fn greet_request(name: &str) -> String {
    //     return serde_json::from_str(
    //         &reqwest::Client::new()
    //         .post("/greet_route")
    //         .body(serde_json::to_string(
    //             &greetData {
    //                 name: name.to_owned(),
    //             }
    //         ).unwrap()).header("Content-Type", "application/json")
    //         .send().await.unwrap()
    //         .text().await.unwrap()
    //     ).unwrap();
    // }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![greet_route])
            .manage(Mutex::new("Robert".to_string()))
            .launch()
            .await
    });

    assert_eq!(
        greet_request(&"Joe".to_string()).await,
        "Hello Joe".to_string()
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}