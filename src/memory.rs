use std::{arch::asm, collections::HashMap, io::{BufReader, Read}, mem::MaybeUninit, os, sync::{Once, PoisonError, RwLock, TryLockError}};

use rocket::tokio;

use crate::{defs::Theme, ServerConfiguration};

#[derive(Debug, Clone)]
pub enum DatabaseError {
    EntryDoesNotExist(String),
    EntryAlreadyExists(String),
    LockIsPoisoned,
    WouldBlock,
    DiskEntryDoesNotExist(String),
    FailedWritingToDisk
}

#[derive(Debug, Clone)]
pub enum Instance {
    Memory(Theme),
    Disk
}


#[derive(Debug)]
pub struct MemoryDatabase {
    loaded_themes: RwLock<HashMap<String, Instance>>
}

impl MemoryDatabase {
    pub fn new () -> Self {
        let mut existing_themes: HashMap<String, Instance> = HashMap::new();
        for i in std::fs::read_dir(ServerConfiguration::singleton().files_path.clone()).unwrap() {
            match i {
                Ok(a) => {
                    existing_themes.insert(a.file_name().into_string().unwrap().replace(".json", ""), Instance::Disk);
                },
                Err(_) => panic!("Could not read files directory")
            }
        }
        return MemoryDatabase {
            loaded_themes: RwLock::new(existing_themes)
        }
    }

    pub async fn start_timer(&self, wait_time: tokio::time::Duration) {
        loop {
            tokio::time::sleep(wait_time).await;
            match self.flush() {
                Ok(_) => (),
                Err(e) => panic!("Encountered error while flushing: {:?}", e)
            }
        }
    }

    pub fn singleton() -> &'static mut MemoryDatabase {
        static mut SINGLETON: MaybeUninit<MemoryDatabase> = MaybeUninit::uninit();
        static ONCE: Once = Once::new();
        
        unsafe {
            ONCE.call_once(|| {
                let singleton = MemoryDatabase::new();
                SINGLETON.write(singleton);
            });

            SINGLETON.assume_init_mut()
        }
    }

    fn peek_disk(&self, name: &String) -> Result<(), ()>{
        match std::fs::File::open(format!("{}/{}.json", ServerConfiguration::singleton().files_path, name.to_lowercase())) {
            Ok(_) => return Ok(()),
            Err(_) => return Err(())
        }
    }

    fn get_from_disk(&self, name: &String) -> Result<Theme, DatabaseError> {
        match std::fs::File::open(format!("{}/{}.json", ServerConfiguration::singleton().files_path, name.to_lowercase())) {
            Ok(mut a) => {
                let mut str: String = String::new();
                a.read_to_string(&mut str);
                
                match serde_json::from_str::<Theme>(str.as_str()) {
                    Ok(a) => {
                        return Ok(a)
                    }, Err(e) => {
                        
                        return Err(DatabaseError::EntryDoesNotExist(name.clone()))
                    }
                }
            }, Err(e) => {
                
                return Err(DatabaseError::DiskEntryDoesNotExist(name.clone()))
            }
        }
    }

    fn insert_to_disk(&self, value: Theme) -> Result<(), ()> {
        match std::fs::write(format!("{}/{}.json", ServerConfiguration::singleton().files_path, value.name.to_lowercase()), serde_json::to_string(&value).unwrap()) {
            Ok(_) => return Ok(()),
            Err(_) => return Err(())
        }
    }

    fn remove_from_disk(&self, name: &String) -> Result<(), ()> {
        match std::fs::remove_file(format!("{}/{}.json", ServerConfiguration::singleton().files_path, name.clone().to_lowercase())) {
            Ok(_) => return Ok(()),
            Err(_) => return Err(())
        }
    }

    pub fn get(&self, name: String) -> Option<Theme> {
        /*
         * If the key is found in the in-memory database, it will be returned
         * else, it will load the data from the disk
         */
        match self.loaded_themes.try_read() {
            Ok(a) => {
                match a.get(&name.to_lowercase()) {
                    Some(a) => {
                        match a  {
                            Instance::Memory(a) => return Some(a.clone()),
                            Instance::Disk => {
                                match self.get_from_disk(&name) {
                                    Ok(a) => return Some(a),
                                    Err(_) => return None
                                }
                            }
                        }
                    },
                    None => return None
                }
            }, Err(_) => return None
         }
    }

    pub fn set(&mut self, name: String, value: Theme) -> Result<(), DatabaseError> {
        match self.loaded_themes.try_write() {
            Ok(mut map) => {
                if !map.contains_key(&name.to_lowercase()) {
                    match self.peek_disk(&name) {
                        Ok(_) => {
                            
                            map.insert(name.to_lowercase(), Instance::Disk);
                        },
                        Err(_) => return Err(DatabaseError::EntryDoesNotExist(name))
                    }
                }
                match map.get(&name.to_lowercase()).unwrap() {
                    Instance::Memory(_) => {
                        
                        map.remove(&name.to_lowercase());
                        map.insert(value.name.to_lowercase(), Instance::Memory(value));
                        return Ok(())
                    }, Instance::Disk => {
                        
                        match self.get_from_disk(&name.to_lowercase()) {
                            Ok(_) => {
                                
                                map.remove(&name.to_lowercase());
                                map.insert(value.name.to_lowercase(), Instance::Memory(value.clone()));
                                return Ok(());
                            }, Err(_) => return Err(DatabaseError::EntryDoesNotExist(name))
                        }
                    }
                }
            },
            Err(e) => {
                match e {
                    TryLockError::Poisoned(_) => return Err(DatabaseError::LockIsPoisoned),
                    TryLockError::WouldBlock => return Err(DatabaseError::WouldBlock)
                }   
            }
        }
    }

    pub fn insert(&mut self, value: Theme) -> Result<(), DatabaseError> {
        let name = value.name.clone();
        match self.loaded_themes.try_write() {
            Ok(mut map) => {
                if map.contains_key(&name.to_lowercase()) { return Err(DatabaseError::EntryAlreadyExists(value.name.to_lowercase())) }
                map.insert(name.to_lowercase(), Instance::Memory(value));
                return Ok(())
            }, Err(e) => {
                match e {
                    TryLockError::Poisoned(_) => return Err(DatabaseError::LockIsPoisoned),
                    TryLockError::WouldBlock => return Err(DatabaseError::WouldBlock)
                }
            }
        }
    }

    pub fn remove(&mut self, name: String) -> Result<(), DatabaseError> {
        match self.loaded_themes.try_write() {
            Ok(mut map) => {
                if !map.contains_key(&name.to_lowercase()) { return Err(DatabaseError::EntryDoesNotExist(name)) }
                match self.remove_from_disk(&name) {
                    Ok(_) => {
                        map.remove(&name.to_lowercase());
                        return Ok(())
                    }, Err(_) => return Err(DatabaseError::FailedWritingToDisk)
                }
            }, Err(e) => {
                match e {
                    TryLockError::Poisoned(_) => return Err(DatabaseError::LockIsPoisoned),
                    TryLockError::WouldBlock => return Err(DatabaseError::WouldBlock)
                }
            }
        }
    }

    pub fn flush(&self) -> Result<(), DatabaseError> {
        match self.loaded_themes.try_read() {
            Ok(map) => {
                for i in std::fs::read_dir(ServerConfiguration::singleton().files_path.clone()).unwrap() {
                    match i {
                        Ok(a) => {
                            if !map.contains_key(&a.file_name().into_string().unwrap().replace(".json", "")) {
                                std::fs::remove_file(format!("{}/{}", ServerConfiguration::singleton().files_path, a.file_name().into_string().unwrap()));
                            }
                        },
                        Err(_) => return Err(DatabaseError::FailedWritingToDisk)
                    }
                }
                for (k, v) in map.iter() {
                    match v {
                        Instance::Disk => continue,
                        Instance::Memory(theme) => {
                            std::fs::write(format!("{}/{}.json", ServerConfiguration::singleton().files_path, k.to_lowercase()), serde_json::to_string(theme).unwrap());
                        }
                    }
                }
                return Ok(())
            }, Err(e) => {
                match e {
                    TryLockError::WouldBlock => return Err(DatabaseError::WouldBlock),
                    TryLockError::Poisoned(_) => return Err(DatabaseError::LockIsPoisoned)
                }
            }
        }
    }
}

#[cfg(test)]
pub mod memory_test {
    use std::time::Duration;

    use rocket::tokio;

    use crate::{defs::Theme, ServerConfiguration};

    use super::{Instance, MemoryDatabase};

    #[test]
    fn db_start_is_ok() {
        let mut db = MemoryDatabase::new();
        
        assert!(db.get("Test".to_string()).is_some());
    }

    #[test]
    fn db_flush_is_ok() {
        let mut db = MemoryDatabase::new();
        db.insert(Theme::new("Test1"));
        
        assert!(db.get("Test1".to_string()).is_some());
        db.flush();
        assert!(std::fs::exists(format!("{}/{}", ServerConfiguration::singleton().files_path, "test1.json")).unwrap());
        
        db.set("Test1".to_string(), Theme::new("Test2"));
        assert!(db.get("Test1".to_string()).is_none());
        
        db.flush();
        assert!(!std::fs::exists(format!("{}/{}", ServerConfiguration::singleton().files_path, "test1.json")).unwrap());
    }
    

    #[tokio::test]
    async fn db_timer_is_ok() {
        MemoryDatabase::singleton().insert(Theme::new("Test5"));
        MemoryDatabase::singleton().insert(Theme::new("Test6"));
        tokio::spawn(async move {
            MemoryDatabase::singleton().start_timer().await;
        });
        
        tokio::time::sleep(tokio::time::Duration::from_secs(6)).await;
        
        
        
    }

    #[test]
    fn get_returns_error() {
        let mut db = MemoryDatabase::new();
        assert!(db.get("CavalinhoPocoto".to_string()).is_none() == true);
    }

    #[test]
    fn insert_is_ok() {
        let mut db = MemoryDatabase::new();
        db.insert(Theme::new("Test"));
        assert!(db.get("Test".to_string()).is_some());
        assert!(db.get("Test".to_string()).unwrap().name == "Test");
    }

    #[test]
    fn set_is_ok() {
        let mut db = MemoryDatabase::new();
        
        db.insert(Theme::new("Test3"));
        assert!(db.get("Test3".to_string()).is_some());
        db.set("Test3".to_string(), Theme::new("Test4"));
        assert!(db.get("Test3".to_string()).is_none());
        assert!(db.get("Test4".to_string()).is_some());
    }
}
