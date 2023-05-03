use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, Write},
    ops::DerefMut,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct DbControl {
    db: Arc<Db>,
}

#[derive(Debug)]
struct Db {
    map: Mutex<HashMap<String, String>>,
}

impl DbControl {
    pub fn new() -> Self {
        if let Ok(map) = read_backup() {
            tracing::info!("read data from backup");
            return Self {
                db: Arc::new(Db {
                    map: Mutex::new(map),
                }),
            };
        }
        Self {
            db: Arc::new(Db {
                map: Mutex::new(HashMap::with_capacity(10)),
            }),
        }
    }

    pub fn set(&self, key: String, value: String) -> Option<String> {
        let mut map = self.db.map.lock().unwrap();
        map.insert(key, value)
    }

    pub fn get(&self, key: &String) -> Option<String> {
        let map = self.db.map.lock().unwrap();
        map.get(key).map(|s| s.clone())
    }

    pub fn del(&self, key: &String) -> Option<String> {
        let mut map = self.db.map.lock().unwrap();
        map.remove(key)
    }

    pub fn backup(self) -> std::io::Result<()> {
        let mut binding = self.db.map.lock().unwrap();
        let mut fs = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("kvs_backup.json")?;
        let buf = serde_json::to_vec(binding.deref_mut())?;

        fs.write_all(&*buf)?;

        fs.flush()?;
        Ok(())
    }
}

fn read_backup() -> std::io::Result<HashMap<String, String>> {
    let backup = File::open("kvs_backup.json")?;
    let reader = BufReader::new(backup);
    let map: HashMap<String, String> = serde_json::from_reader(reader)?;
    Ok(map)
}
