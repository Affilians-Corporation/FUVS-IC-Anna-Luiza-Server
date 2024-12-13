use defs::{Theme, SubTheme};
use rocket::{fs::FileServer, http::Status};
use rocket::serde::json::Json;

pub mod defs;

#[macro_use] extern crate rocket;

const FILES_PATH: &'static str = r"C:\Users\matteusmaximo\Documents\anna-luiza-server\files";

#[get("/theme/<name>")]
fn theme(name: &str) -> String {
    for i in std::fs::read_dir(FILES_PATH).unwrap() {
        match i {
            Ok(a) => {
                if a.file_name().to_str().unwrap() == name {
                    let str_theme = std::fs::read_to_string(format!("{}/{}", FILES_PATH, a.file_name().to_str().unwrap())).unwrap();
                    return str_theme
                }
            }, Err(_) => continue
        }
    }
 
    return "Internal Server Error".to_string()
}

#[post("/new_theme", data = "<input>")]
fn put_theme(input: Json<Theme>) -> Status {
    println!("{:?}", input);
    return Status::Ok
}

#[get("/")]
fn index() -> &'static str {
    "Hello, World!"
}

#[launch]
fn rocket() -> _{
    rocket::build()
        .mount("/", routes![index, theme, put_theme])
        .mount("/res", FileServer::from(FILES_PATH))
}
