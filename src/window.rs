extern crate failure;

extern crate serde;
extern crate serde_json;

use std::io::{Read, Write};
use std::fs::File;
use self::failure::Error;

pub type Point = (i32, i32);

#[derive(Serialize, Deserialize)]
pub struct Window {
    pub portrait_bounds: (Point, Point),
    pub landscape_bounds: (Point, Point),
    pub paintbrush: Point,
    pub spray_can: Point,
    pub pen: Point,
    pub change_orientation: Point,
}

impl Window {
    pub fn new(path: String) -> Result<Window, Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let win: Window = serde_json::from_str(&contents)?;
        Ok(win)
    }

    pub fn save(&self, path: String) -> Result<(), Error> {
        let j = serde_json::to_string_pretty(self)?;
        let mut file = File::create(&path)?;
        file.write_all(j.as_bytes())?;
        Ok(())
    }
}
