mod model;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use model::{User, UserLogin};
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};

const DB_NAME: &str = "myApp";
const COLL_NAME: &str = "users";


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let uri = std::env::var("mongodb://localhost:27017").unwrap_or_else(|_| "mongodb://localhost:27017".into());

    let client = Client::with_uri_str(uri).await.expect("failed to connect");
    create_username_index(&client).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .route("/", web::get().to(index))
            .route("/login", web::get().to(login))
            .service(verification)
            .service(add_user)
	        .service(get_user)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn index() -> HttpResponse {
    let html = r#"<html>
        <head><title>User Test</title></head>
        <body>
        <form action=/add_user method=POST>
            <label>
                First Name:
                <input name="first_name">
            </label>
            <label>
                Last Name:
                <input name="last_name">
            </label>
            <label>
                Username:
                <input name="username">
            </label>
            <label>
                Password:
                <input name="password", type = "password">
            </label>
            <label>
                Email:
                <input name="email", type = "email">
            </label>
            <button type=submit>Create User</button>
        </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

async fn login() -> HttpResponse {    
    let loginhtml = r#"<html>
        <head><title>User Test</title></head>
            <body>
            <form action=/verification method=POST>
                <label>
                    Username:
                    <input name="username">
                </label>
                <label>
                    Password:
                    <input name="password", type = "password">
                </label>
                <button type=submit>Login</button>
            </form>
            </body>
        </html>"#;
    HttpResponse::Ok().body(loginhtml)
}


#[post("/verification")]
async fn verification(client: web::Data<Client>, form: web::Form<UserLogin>) -> HttpResponse {
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    let logininfo = form.into_inner();
    match collection
    .find_one(doc! { "username": &logininfo.username }, None)
    .await
{
    Ok(Some(user)) if user.password.ne(&logininfo.password) => {
        HttpResponse::NotFound().body(format!("Invalid password for user: {}", user.username))
    }
    Ok(Some(user)) => HttpResponse::Ok().json(user),
    Ok(None) => {
        HttpResponse::NotFound().body(format!("No user found with username {}", logininfo.username))
    }

    Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
}
}

/// Adds a new user to the "users" collection in the database.
#[post("/add_user")]
async fn add_user(client: web::Data<Client>, form: web::Form<User>) -> HttpResponse {
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_one(form.into_inner(), None).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("Successfully added user"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Gets the user with the supplied username.
#[get("/get_user/{username}")]
async fn get_user(client: web::Data<Client>, username: web::Path<String>) -> HttpResponse {
    let username = username.into_inner();
    let collection: Collection<User> = client.database(DB_NAME).collection(COLL_NAME);
    match collection
        .find_one(doc! { "username": &username }, None)
        .await
    {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No user found with username {username}"))
        }
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

/// Creates an index on the "username" field to force the values to be unique.
async fn create_username_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "username": 1 })
        .options(options)
        .build();
    client
        .database(DB_NAME)
        .collection::<User>(COLL_NAME)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}


