use retrofit_codegen::api;

#[tokio::test]
#[serial_test::serial]
async fn test_api() {
    #[api]
    fn plus(num1: i32, num2: i32) -> i32 {
        num1 + num2
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new().route("/plus", axum::routing::post(plus_route));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let input1 = 10;
    let input2 = 100;
    // Call request
    let result = plus_request(input1, input2).await.unwrap();
    assert_eq!(result, plus(input1, input2));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
#[serial_test::serial]
async fn test_api_vec() {
    #[api]
    fn sum(nums: Vec<i32>) -> i32 {
        nums.iter().sum()
    }

    #[api]
    fn hello_world() {
        println!("Hello World!");
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new()
            .route("/sum", axum::routing::post(sum_route))
            .route("/hello_world", axum::routing::post(hello_world_route));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let inputs = vec![10, 123, 4354];
    // Call request
    let result = sum_request(inputs.clone()).await.unwrap();
    assert_eq!(result, sum(inputs));

    hello_world_request().await.unwrap();

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
#[serial_test::serial]
async fn test_api_state() {
    // Test a GET API with a managed state
    #[api(std::sync::Arc<std::sync::Mutex<String>>)]
    fn greet(name: String, state: &std::sync::Arc<std::sync::Mutex<String>>) -> String {
        let mut state = state.lock().unwrap();
        let greeting = format!("Hello {name}, I'm here with {state}");
        *state = name;
        greeting
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new()
            .route("/greet", axum::routing::post(greet_route))
            .with_state(std::sync::Arc::new(std::sync::Mutex::new(
                "Robert".to_string(),
            )));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    assert_eq!(
        greet_request("Joe".to_string()).await.unwrap(),
        "Hello Joe, I'm here with Robert".to_string()
    );
    assert_eq!(
        greet_request("Frank".to_string()).await.unwrap(),
        "Hello Frank, I'm here with Joe".to_string()
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
#[serial_test::serial]
async fn test_api_ref() {
    // Test POST API with references
    #[api]
    fn greet(nm: &String, num: i32) -> String {
        format!("Hello {nm}{num}")
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        let app = axum::Router::new().route("/greet", axum::routing::post(greet_route));
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let name = "Gordon Shumway".to_string();
    assert_eq!(greet_request(&name, 23).await.unwrap(), greet(&name, 23));

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
#[serial_test::serial]
async fn test_routes_module() {
    #[crate::routes_module]
    mod functions {
        // Test POST API with references
        #[crate::api]
        pub fn greet(nm: &String, num: i32) -> String {
            format!("Hello {nm}{num}")
        }

        #[crate::api(std::sync::Arc<std::sync::Mutex<String>>)]
        pub fn greet2(name: String, state: &std::sync::Arc<std::sync::Mutex<String>>) -> String {
            let mut state = state.lock().unwrap();
            let greeting = format!("Hello {name}, I'm here with {state}");
            *state = name;
            greeting
        }

        #[crate::api(std::sync::Arc<std::sync::Mutex<String>>)]
        pub fn greet3(name: String, state: &std::sync::Arc<std::sync::Mutex<String>>) -> String {
            let mut state = state.lock().unwrap();
            let greeting = format!("Hello {name}, I'm here with {state}");
            *state = name;
            greeting
        }
    }

    // Launch server
    let server_handle = tokio::spawn(async {
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(
                functions::routes()
                    .with_state(std::sync::Arc::new(std::sync::Mutex::new(
                        "Robert".to_string(),
                    )))
                    .into_make_service(),
            )
            .await
            .unwrap();
    });

    let name = "Gordon Shumway".to_string();
    assert_eq!(
        functions::greet_request(&name, 23).await.unwrap(),
        functions::greet(&name, 23)
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}

#[tokio::test]
#[serial_test::serial]
async fn test_routes() {
    mod functions {
        crate::routes! {
            // Test POST API with references
            #[crate::api]
            pub fn greet(nm: &String, num: i32) -> String {
                format!("Hello {nm}{num}")
            }

            #[crate::api(std::sync::Arc<std::sync::Mutex<String>>)]
            pub fn greet2(name: String, state: &std::sync::Arc<std::sync::Mutex<String>>) -> String {
                let mut state = state.lock().unwrap();
                let greeting = format!("Hello {name}, I'm here with {state}");
                *state = name;
                greeting
            }

            #[crate::api(std::sync::Arc<std::sync::Mutex<String>>)]
            pub fn greet3(name: String, state: &std::sync::Arc<std::sync::Mutex<String>>) -> String {
                let mut state = state.lock().unwrap();
                let greeting = format!("Hello {name}, I'm here with {state}");
                *state = name;
                greeting
            }
        }
    }

    // Launch server
    let router = functions::routes().with_state(std::sync::Arc::new(std::sync::Mutex::new(
        "Robert".to_string(),
    )));
    let server_handle = tokio::spawn(async {
        axum::Server::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 8000)))
            .serve(router.into_make_service())
            .await
            .unwrap();
    });

    let name = "Gordon Shumway".to_string();
    assert_eq!(
        functions::greet_request(&name, 23).await.unwrap(),
        functions::greet(&name, 23)
    );

    server_handle.abort();
    assert!(server_handle.await.unwrap_err().is_cancelled());
}
