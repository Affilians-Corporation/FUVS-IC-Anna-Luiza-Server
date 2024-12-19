use std::mem::MaybeUninit;
use std::sync::Once;
use defs::{Theme, SubTheme};
use rocket::response::{content, status};
use rocket::{fs::FileServer, http::Status};
use rocket::serde::json::Json;
use memory::{ MemoryDatabase, DatabaseError };
use serde::Deserialize;

pub mod defs;
pub mod memory;

#[macro_use] extern crate rocket;

// const FILES_PATH: &'static str = r"C:\Users\matteusmaximo\Documents\anna-luiza-server\files";
// const FLUSH_TIME: tokio::time::Duration = tokio::time::Duration::from_secs(5);

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfiguration {
    pub files_path: String,
    pub flush_frequency: u64
}

impl ServerConfiguration {
    pub fn new(path: &str) -> Self {
        let config_str = std::fs::read_to_string(path).unwrap();
        return toml::from_str(config_str.as_str()).unwrap()
    }

    pub fn get_flush_frequency(&self) -> tokio::time::Duration {
        return tokio::time::Duration::from_secs(self.flush_frequency)
    }

    pub fn singleton() -> &'static ServerConfiguration{
        static mut SINGLETON: MaybeUninit<ServerConfiguration> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();
        unsafe {
            ONCE.call_once(|| {
                let singleton = ServerConfiguration::new(std::env::var("IC_CONFIG_PATH").unwrap().as_str());
                SINGLETON.write(singleton);
            });
            SINGLETON.assume_init_ref()
        }
    }
}

#[get("/theme/<name>")]
fn theme(name: &str) -> status::Custom<content::RawJson<String>> {
    println!("Request Received");
    match MemoryDatabase::singleton().get(name.to_string()) {
        Some(a) => {
            match serde_json::to_string(&a) {
                Ok(j) => return status::Custom(Status::Ok, content::RawJson(j)),
                Err(_) => return status::Custom(Status::InternalServerError, content::RawJson("{\"msg\": \"Error parsing json\"}".to_string()))
            }
        }, None => {
            return status::Custom(Status::NotFound, content::RawJson("\"msg\": \"Theme not found in database\"".to_string()))
        }
    }
}

#[post("/theme", data = "<input>")]
fn set_theme(input: Json<Theme>) -> status::Custom<content::RawJson<String>> {
   let parsed_theme: Theme = input.into_inner();
   match MemoryDatabase::singleton().set(parsed_theme.name.clone(), parsed_theme) {
       Ok(_) => return status::Custom(Status::Ok, content::RawJson("".to_string())),
       Err(e) => {
           match e {
               DatabaseError::WouldBlock => return status::Custom(
                                                Status::InternalServerError, 
                                                content::RawJson("{\"reason\": \"WouldBlock\"}".to_string())
                                            ),
               DatabaseError::LockIsPoisoned => return status::Custom(
                                                Status::InternalServerError, 
                                                content::RawJson("{\"reason\": \"Poisoned\"}".to_string())
                                            ),
               DatabaseError::EntryDoesNotExist(_) => return status::Custom(
                                                Status::InternalServerError, 
                                                content::RawJson("{\"reason\": \"EntryDoesNotExist\"}".to_string())
                                            ),
               DatabaseError::DiskEntryDoesNotExist(_) => return status::Custom(
                                                Status::InternalServerError,
                                                content::RawJson("{\"reason\": \"DiskEntryDoesNotExist\"}".to_string())
                                            ),
                                            _ => return status::Custom(
                                                Status::InternalServerError, 
                                                content::RawJson("{\"reason\": \"Unknown\"}".to_string())
                                            )
           }
       }
   }
}

#[put("/theme", data = "<input>")]
fn put_theme(input: Json<Theme>) -> status::Custom<content::RawJson<String>> {
    let parsed_theme: Theme = input.into_inner();
    match MemoryDatabase::singleton().insert(parsed_theme) {
        Ok(_) => return status::Custom(Status::Ok, content::RawJson("".to_string())),
        Err(e) => {
            match e {
                DatabaseError::WouldBlock => return status::Custom(
                                                Status::InternalServerError,
                                                content::RawJson("{\"reason\": \"WouldBlock\"}".to_string())
                                            ),
                DatabaseError::LockIsPoisoned => return status::Custom( 
                                                Status::InternalServerError, 
                                                content::RawJson("{\"reason\": \"Poisoned\"}".to_string()) 
                                            ),
                DatabaseError::EntryAlreadyExists(_) => return status::Custom(
                                                Status::InternalServerError, 
                                                content::RawJson("{\"reason\": \"EntryAlreadyExists\"}".to_string())
                                            ),
                                            _ => return status::Custom ( 
                                                Status::InternalServerError, 
                                                content::RawJson("{\"reason\": \"Unknown\"}".to_string())
                                            )
            }
        }
    }
}

/* #[delete("/theme/<name>")]
fn del_theme(name: &str) -> Status {
    let res = std::fs::remove_file(format!("{}/{}.json", FILES_PATH, name.to_lowercase()));
    match res {
        Ok(_) => return Status::Ok,
        Err(_) => return Status::InternalServerError
    }
}*/

#[get("/")]
fn index() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let db = MemoryDatabase::singleton();
    tokio::spawn(async move {
        MemoryDatabase::singleton().start_timer(ServerConfiguration::singleton().get_flush_frequency()).await;
    });
    db.insert(Theme::new("Test15"));
    db.insert(Theme::new("Test39"));

    println!("{:?}", db);

    rocket::build()
        .mount("/", routes![index, theme, put_theme, set_theme])
        .mount("/res", FileServer::from(ServerConfiguration::singleton().files_path.clone())).launch().await;
}
