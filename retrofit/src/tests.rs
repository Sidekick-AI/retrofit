pub use reqwasm;
pub use reqwest;
use retrofit_codegen::{get_api, post_api};
pub use rocket;
use rocket::routes;
use std::sync::Mutex;

#[cfg(feature="rocket")]
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

#[cfg(feature="rocket")]
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

#[cfg(feature="rocket")]
#[tokio::test]
#[serial_test::serial]
async fn test_get_api_ref() {
    // Test a normal GET API
    #[get_api]
    fn greet(name: &String) -> String {
        format!("Hello {}", name)
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![greet_route])
            .launch()
            .await
    });

    let name = "Sheila".to_string();
    // Call request
    let result = greet_request(&name).await;
    assert_eq!(result, greet(&name));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="rocket")]
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

#[cfg(feature="rocket")]
#[tokio::test]
#[serial_test::serial]
async fn test_post_api_vec() {
    #[post_api]
    fn sum(nums: Vec<i32>) -> i32 {
        nums.iter().sum()
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![sum_route])
            .launch()
            .await
    });

    let inputs = vec![10, 123, 4354];
    // Call request
    let result = sum_request(inputs.clone()).await;
    assert_eq!(result, sum(inputs));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="rocket")]
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

#[cfg(feature="rocket")]
#[tokio::test]
#[serial_test::serial]
async fn test_post_api_ref() {
    // Test POST API with references
    #[post_api]
    fn greet(nm: &String, num: i32) -> String {
        format!("Hello {}{}", nm, num)
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![greet_route])
            .manage(Mutex::new("Robert".to_string()))
            .launch()
            .await
    });

    let name = "Gordon Shumway".to_string();
    assert_eq!(
        greet_request(&name, 23).await,
        greet(&name, 23)
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}