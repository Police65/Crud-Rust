#[macro_use] extern crate rocket;

use rocket::response::Redirect;
use rocket_dyn_templates::Template;
use rocket::fs::FileServer;
use rocket::fs::relative;
use rocket_db_pools::{Database, sqlx};
use rocket_db_pools::sqlx::Row;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Database)]
#[database("products")]
struct Products(sqlx::SqlitePool);

#[derive(Serialize)]
struct Product {
    id: i32,
    name: String,
    price: f32, 
    quantity: i32,
}

#[get("/")]
async fn index(db: &Products) -> Template {
    let mut context = HashMap::new();
    let rows = sqlx::query("SELECT * FROM products")
        .fetch_all(&db.0)
        .await
        .unwrap();
    let products: Vec<Product> = rows.into_iter().map(|row| {
        Product {
            id: row.get::<i32, _>("id"),
            name: row.get::<String, _>("name"),
            price: row.get::<f32, _>("price"),
            quantity: row.get::<i32, _>("quantity"),
        }
    }).collect();
    context.insert("products", products);
    Template::render("index", &context)
}

#[get("/crear")]
fn crear() -> Template {
    let context = HashMap::<&str, &str>::new();
    Template::render("crear", &context)
}

#[derive(FromForm)]
struct ProductForm {
    name: String,
    price: f32,
    quantity: i32,
}

#[post("/add", data = "<form>")]
async fn crear_producto(form: rocket::form::Form<ProductForm>, db: &Products) -> Result<Redirect, Template> {
    let mut context = HashMap::new();
    let result = sqlx::query("INSERT INTO products (name, price, quantity) VALUES (?, ?, ?)")
        .bind(&form.name)
        .bind(&form.price)
        .bind(&form.quantity)
        .execute(&db.0)
        .await;
     match result {
        Ok(_) => {
            Ok(Redirect::to("/"))
        },
        Err(_) => {
            context.insert("message", "Error al crear el producto");
            Err(Template::render("crear", &context))
        }
    }
}

#[get("/editar/<id>")]
async fn editar(id: i32, db: &Products) -> Template {
    let mut context = HashMap::new();
    let row = sqlx::query("SELECT * FROM products WHERE id = ?")
        .bind(id)
        .fetch_one(&db.0)
        .await
        .unwrap();
    let product = Product {
        id: row.get::<i32, _>("id"),
        name: row.get::<String, _>("name"),
        price: row.get::<f32, _>("price"),
        quantity: row.get::<i32, _>("quantity"),
    };
    context.insert("product", product);
    Template::render("editar", &context)
}

#[derive(FromForm)]
struct EditProductForm {
    id: i32,
    name: String,
    price: f32,
    quantity: i32,
}

#[post("/update", data = "<form>")]
async fn update(form: rocket::form::Form<EditProductForm>, db: &Products) -> Result<Redirect, Template> {
    let mut context = HashMap::new();
    let result = sqlx::query("UPDATE products SET name = ?, price = ?, quantity = ? WHERE id = ?")
        .bind(&form.name)
        .bind(&form.price)
        .bind(&form.quantity)
        .bind(&form.id)
        .execute(&db.0)
        .await;
     match result {
        Ok(_) => {
            Ok(Redirect::to("/"))
        },
        Err(_) => {
            context.insert("message", "Error al actualizar el producto");
            Err(Template::render("editar", &context))
        }
    }
}

#[get("/borrar/<id>")]
async fn borrar(id: i32, db: &Products) -> Result<Redirect, Template> {
    let mut context = HashMap::new();
    let result = sqlx::query("DELETE FROM products WHERE id = ?")
        .bind(id)
        .execute(&db.0)
        .await;
     match result {
        Ok(_) => {
            Ok(Redirect::to("/"))
        },
        Err(_) => {
            context.insert("message", "Error al borrar el producto");
            Err(Template::render("editar", &context))
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
    .mount("/static", FileServer::from(relative!("/static")))
    .mount("/", routes![index, crear, crear_producto, editar, update, borrar])
    .attach(Template::fairing())
    .attach(Products::init())
}
