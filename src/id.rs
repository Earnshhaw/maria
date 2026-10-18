use crate::dirs::app_data_dir;
use std::fs::read_dir;
use uuid::Uuid;

pub fn ensure_unique_id() -> Uuid {
    let mut id = Uuid::new_v4();
    let a = read_dir(app_data_dir());
    match a {
        Ok(x) => {
            let x: Vec<String> = x
                .filter_map(|e| e.ok().map(|x| x.file_name().to_string_lossy().into_owned()))
                .collect();
            while x.iter().any(|uuid| *uuid == id.to_string()) {
                id = Uuid::new_v4();
            }
            id
        }
        Err(e) => {
            eprintln!("{e}");
            return Uuid::new_v4();
        }
    }
}
