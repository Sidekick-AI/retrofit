pub use retrofit_codegen::get_api;

#[cfg(test)]
mod tests {
    pub use rocket;
    pub use reqwest;
    pub use reqwasm;
    use retrofit_codegen::get_api;
    use rocket::routes;

    #[tokio::test]
    async fn test_get_api() {
        #[get_api]
        fn plus(num1: i32, num2: i32) -> i32 {
            num1 + num2
        }

        // Launch server
        tokio::spawn(async {
            rocket::build().mount("/", routes![plus_route]).launch().await
        });

        let input1 = 10;
        let input2 = 100;
        // Call request
        let result = plus_request(input1, input2).await;
        assert_eq!(result, plus(input1, input2));
    }
}
