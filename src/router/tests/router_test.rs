use rocket::http::Status;
use rocket::local::asynchronous::Client;


#[rocket::async_test]
async fn hello_test() {
    let client = Client::tracked(router::rocket().await).await.expect("valid rocket instance");
    let req = client.get("/");
    let response = rocket::tokio::join!(req.clone().dispatch());
    assert_eq!(response.0.status(), Status::Ok);
    assert_eq!(response.0.into_string().await.unwrap(), r#"{"code":200,"description":"hello world!"}"#)
}