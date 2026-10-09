use std::fs::read_dir;
use uuid::Uuid;

use crate::dirs::app_data_dir;

pub fn ensure_unique_id() -> Uuid {
    let mut id = Uuid::new_v4();
    let x: Vec<String> = read_dir(app_data_dir().unwrap()).unwrap().filter_map(|e| e.ok().map(|x| x.file_name().to_string_lossy().into_owned())).collect();
    while x.iter().any(|uuid| *uuid == id.to_string()) {
        id = Uuid::new_v4();
    }
    id
}

