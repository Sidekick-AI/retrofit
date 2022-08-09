use retrofit_codegen::{get_api, post_api};

#[cfg(feature="rocket")]
use rocket::{self, routes};

#[cfg(feature="rocket")]
#[tokio::test]
#[serial_test::serial]
async fn test_rocket_get_api() {
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
async fn test_rocket_get_api_state() {
    // Test a GET API with a managed state
    #[get_api(std::sync::Mutex<String>)]
    fn greet(name: String, state: &std::sync::Mutex<String>) -> String {
        let mut state = state.lock().unwrap();
        let greeting = format!("Hello {}, I'm here with {}", name, state);
        *state = name;
        greeting
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![greet_route])
            .manage(std::sync::Mutex::new("Robert".to_string()))
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
async fn test_rocket_get_api_ref() {
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
async fn test_rocket_post_api() {
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
async fn test_rocket_post_api_vec() {
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
async fn test_rocket_post_api_state() {
    // Test a GET API with a managed state
    #[post_api(std::sync::Mutex<String>)]
    fn greet(name: String, state: &std::sync::Mutex<String>) -> String {
        let mut state = state.lock().unwrap();
        let greeting = format!("Hello {}, I'm here with {}", name, state);
        *state = name;
        greeting
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![greet_route])
            .manage(std::sync::Mutex::new("Robert".to_string()))
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
async fn test_rocket_post_api_ref() {
    // Test POST API with references
    #[post_api]
    fn greet(nm: &String, num: i32) -> String {
        format!("Hello {}{}", nm, num)
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", routes![greet_route])
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

#[cfg(feature="rocket")]
#[tokio::test]
#[serial_test::serial]
async fn test_rocket_routes_module() {
    #[crate::routes_module]
    mod functions {
        // Test POST API with references
        #[crate::post_api]
        pub fn greet(nm: &String, num: i32) -> String {
            format!("Hello {}{}", nm, num)
        }
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", functions::routes())
            .manage(std::sync::Mutex::new("Robert".to_string()))
            .launch()
            .await
    });

    let name = "Gordon Shumway".to_string();
    assert_eq!(
        functions::greet_request(&name, 23).await,
        functions::greet(&name, 23)
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="rocket")]
#[tokio::test]
#[serial_test::serial]
async fn test_rocket_routes() {
    mod functions {
        crate::routes! {
            // Test POST API with references
            #[crate::post_api]
            pub fn greet(nm: &String, num: i32) -> String {
                format!("Hello {}{}", nm, num)
            }
        }
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        rocket::build()
            .mount("/", functions::routes())
            .manage(std::sync::Mutex::new("Robert".to_string()))
            .launch()
            .await
    });

    let name = "Gordon Shumway".to_string();
    assert_eq!(
        functions::greet_request(&name, 23).await,
        functions::greet(&name, 23)
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="axum")]
#[tokio::test]
#[serial_test::serial]
async fn test_axum_get_api() {
    // Test a normal GET API
    #[get_api]
    fn plus(num1: i32, num2: i32) -> i32 {
        num1 + num2
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new()
            .route("/plus", axum::routing::get(plus_route));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await.unwrap();
    });

    let input1 = 10;
    let input2 = 100;
    // Call request
    let result = plus_request(input1, input2).await;
    assert_eq!(result, plus(input1, input2));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="axum")]
#[tokio::test]
#[serial_test::serial]
async fn test_axum_get_api_state() {
    // Test a GET API with a managed state
    #[get_api(std::sync::Arc<std::sync::Mutex<String>>)]
    fn greet(name: String, state: &std::sync::Arc<std::sync::Mutex<String>>) -> String {
        let name = name.replace('"', "");
        let mut state = state.lock().unwrap();
        let greeting = format!("Hello {}, I'm here with {}", name, state);
        *state = name;
        greeting
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new()
            .route("/greet", axum::routing::get(greet_route))
            .layer(axum::Extension(std::sync::Arc::new(std::sync::Mutex::new("Robert".to_string()))));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await.unwrap();
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

#[cfg(feature="axum")]
#[tokio::test]
#[serial_test::serial]
async fn test_axum_get_api_ref() {
    // Test a normal GET API
    #[get_api]
    fn greet(name: &String) -> String {
        format!("Hello {}", name.replace('"', ""))
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new()
            .route("/greet", axum::routing::get(greet_route));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await.unwrap();
    });

    let name = "Sheila".to_string();
    // Call request
    let result = greet_request(&name).await;
    assert_eq!(result, greet(&name));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="axum")]
#[tokio::test]
#[serial_test::serial]
async fn test_axum_post_api() {
    #[post_api]
    fn plus(num1: i32, num2: i32) -> i32 {
        num1 + num2
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new()
            .route("/plus", axum::routing::post(plus_route));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await.unwrap();
    });

    let input1 = 10;
    let input2 = 100;
    // Call request
    let result = plus_request(input1, input2).await;
    assert_eq!(result, plus(input1, input2));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="axum")]
#[tokio::test]
#[serial_test::serial]
async fn test_axum_post_api_vec() {
    #[post_api]
    fn sum(nums: Vec<i32>) -> i32 {
        nums.iter().sum()
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new()
            .route("/sum", axum::routing::post(sum_route));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await.unwrap();
    });

    let inputs = vec![10, 123, 4354];
    // Call request
    let result = sum_request(inputs.clone()).await;
    assert_eq!(result, sum(inputs));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="axum")]
#[tokio::test]
#[serial_test::serial]
async fn test_axum_post_api_state() {
    // Test a GET API with a managed state
    #[post_api(std::sync::Arc<std::sync::Mutex<String>>)]
    fn greet(name: String, state: &std::sync::Arc<std::sync::Mutex<String>>) -> String {
        let mut state = state.lock().unwrap();
        let greeting = format!("Hello {}, I'm here with {}", name, state);
        *state = name;
        greeting
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new()
            .route("/greet", axum::routing::post(greet_route))
            .layer(axum::Extension(std::sync::Arc::new(std::sync::Mutex::new("Robert".to_string()))));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await.unwrap();
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

#[cfg(feature="axum")]
#[tokio::test]
#[serial_test::serial]
async fn test_axum_post_api_ref() {
    // Test POST API with references
    #[post_api]
    fn greet(nm: &String, num: i32) -> String {
        format!("Hello {}{}", nm, num)
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new()
            .route("/greet", axum::routing::post(greet_route));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await.unwrap();
    });

    let name = "Gordon Shumway".to_string();
    assert_eq!(
        greet_request(&name, 23).await,
        greet(&name, 23)
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="axum")]
#[tokio::test]
#[serial_test::serial]
async fn test_axum_routes_module() {
    #[crate::routes_module]
    mod functions {
        // Test POST API with references
        #[crate::post_api]
        pub fn greet(nm: &String, num: i32) -> String {
            format!("Hello {}{}", nm, num)
        }
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(functions::routes().into_make_service())
            .await.unwrap();
    });

    let name = "Gordon Shumway".to_string();
    assert_eq!(
        functions::greet_request(&name, 23).await,
        functions::greet(&name, 23)
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[cfg(feature="axum")]
#[tokio::test]
#[serial_test::serial]
async fn test_axum_routes() {
    mod functions {
        crate::routes! {
            // Test POST API with references
            #[crate::post_api]
            pub fn greet(nm: &String, num: i32) -> String {
                format!("Hello {}{}", nm, num)
            }
        }
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(functions::routes().into_make_service())
            .await.unwrap();
    });

    let name = "Gordon Shumway".to_string();
    assert_eq!(
        functions::greet_request(&name, 23).await,
        functions::greet(&name, 23)
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}