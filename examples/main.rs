use vk_auth::Authorizer;

#[tokio::main]
async fn main() {
    let auth = Authorizer::builder()
        .with_client(
            reqwest::Client::builder()
                // if you use custom client you need to ensure that cookie_store is enabled.
                .cookie_store(true)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    let token = auth
        .get_token(env!("APP_ID"), env!("EMAIL"), env!("PASSWORD"))
        .await
        .unwrap();
    println!("{:?}", token);
}
