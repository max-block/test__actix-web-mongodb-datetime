use futures::TryStreamExt;

use actix_web::{
    get, post,
    web::{Data, Json},
    App, HttpServer, Responder,
};
use chrono::{Utc, DateTime};
use mongodb::{
    bson::{oid::ObjectId, doc},
    Client, Collection,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,

    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    created_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug)]
struct CreateUserParams {
    name: String,
}

#[get("/users")]
async fn get_users(collection: Data<Collection<User>>) -> impl Responder {
    let mut users: Vec<User> = vec![];
    let mut cursor = collection.find(doc!{}, None).await.unwrap();
    while let Some(user) = cursor.try_next().await.unwrap() {
        users.push(user);
    }
    serde_json::to_string(&users)
}

#[post("/users")]
async fn create_user(
    collection: Data<Collection<User>>,
    params: Json<CreateUserParams>,
) -> impl Responder {
    let new_user = User {
        id: None,
        name: params.name.clone(),
        created_at: Utc::now(),
    };
    collection.insert_one(new_user, None).await.unwrap();
    "create_user"
}

#[actix_web::main]
async fn main() {
    let database_name = "test__actix-web-mongodb-datetime";
    let client = Client::with_uri_str("mongodb://localhost/").await.unwrap();
    let database = client.database(database_name);
    let collection = database.collection::<User>("users");
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(collection.clone()))
            .service(get_users)
            .service(create_user)
    })
    .bind("127.0.0.1:3000")
    .unwrap()
    .run()
    .await
    .unwrap()
}
