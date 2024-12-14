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

#[post("/theme", data = "<input>")]
fn put_theme(input: Json<Theme>) -> Status {
    let theme = input.into_inner();
    let res = std::fs::write(format!("{}/{}.json", FILES_PATH, theme.name.to_lowercase()), serde_json::to_string(&theme).unwrap());
    match res {
        Ok(_) => return Status::Ok,
        Err(_) => return Status::InternalServerError
    }
}

#[delete("/theme/<name>")]
fn del_theme(name: &str) -> Status {
    let res = std::fs::remove_file(format!("{}/{}.json", FILES_PATH, name.to_lowercase()));
    match res {
        Ok(_) => return Status::Ok,
        Err(_) => return Status::InternalServerError
    }
}

#[get("/")]
fn index() -> &'static str {
    "Hello, World!"
}

#[launch]
fn rocket() -> _{
    rocket::build()
        .mount("/", routes![index, theme, put_theme, del_theme])
        .mount("/res", FileServer::from(FILES_PATH))
}
