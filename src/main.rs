mod entity;

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use sea_orm::{Database, DatabaseConnection, EntityTrait, Set};
use entity::user::{self, Entity as User};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use dotenv::dotenv;
use std::env;

#[derive(Serialize, Deserialize)]
struct NewUser {
    name: String,
    email: String,
}

#[derive(Serialize, Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<String>,
}

async fn create_user(db: web::Data<DatabaseConnection>, new_user: web::Json<NewUser>) -> impl Responder {
    let new_user = new_user.into_inner();
    let user = user::ActiveModel {
        name: Set(new_user.name),
        email: Set(new_user.email),
        ..Default::default()
    };

    let res = User::insert(user).exec(db.get_ref()).await;

    match res {
        Ok(user) => HttpResponse::Ok().json(user.last_insert_id),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

async fn get_users(db: web::Data<DatabaseConnection>) -> impl Responder {
    let users = User::find().all(db.get_ref()).await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

async fn update_user(db: web::Data<DatabaseConnection>, user_id: web::Path<Uuid>, user_data: web::Json<UpdateUser>) -> impl Responder {
    let user = User::find_by_id(user_id.into_inner()).one(db.get_ref()).await;

    match user {
        Ok(Some(mut user)) => {
            if let Some(name) = &user_data.name {
                user.name = Set(name.clone());
            }
            if let Some(email) = &user_data.email {
                user.email = Set(email.clone());
            }

            let res = user.update(db.get_ref()).await;

            match res {
                Ok(user) => HttpResponse::Ok().json(user),
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            }
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

async fn delete_user(db: web::Data<DatabaseConnection>, user_id: web::Path<Uuid>) -> impl Responder {
    let user = User::find_by_id(user_id.into_inner()).one(db.get_ref()).await;

    match user {
        Ok(Some(user)) => {
            let res = user.delete(db.get_ref()).await;

            match res {
                Ok(_) => HttpResponse::Ok().finish(),
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            }
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(&database_url).await.expect("Failed to connect to database");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .route("/users", web::post().to(create_user))
            .route("/users", web::get().to(get_users))
            .route("/users/{id}", web::put().to(update_user))
            .route("/users/{id}", web::delete().to(delete_user))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
