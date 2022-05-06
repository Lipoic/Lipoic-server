#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use mongodb::{bson::doc, options::ClientOptions, Client};
use rocket::Request;
use rocket_contrib::json::Json;

mod data;
mod util;

use crate::data::response::Response;

extern crate dotenv;

use dotenv::dotenv;
use std::env;

#[catch(404)]
fn not_found(req: &Request) -> Json<Response> {
    Json(Response::not_found(req))
}

#[get("/teapot")]
fn teapot() -> Json<Response> {
    Json(Response::teapot(&Some("I'm a teapot!")))
}

#[get("/")]
fn index() -> Json<Response> {
    Json(Response::ok(&Some("hello world!")))
}

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    // load .env file
    dotenv().ok();
    let mongodb_url: String = env::var("MONGODB_URL").unwrap();
    let mut client_options = ClientOptions::parse(mongodb_url).await?;

    // Manually set an option
    client_options.app_name = Some("Lipoic Server".to_string());

    // Get a handle to the cluster
    let client = Client::with_options(client_options)?;

    // Ping the server to see if you can connect to the cluster
    client
        .database("admin")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    println!("Connected successfully.");

    // List the names of the databases in that cluster
    for db_name in client.list_database_names(None, None).await? {
        println!("{}", db_name);
    }

    rocket::ignite()
        .register(catchers![not_found])
        .mount("/", routes![index, teapot])
        .launch();

    Ok(())
}
