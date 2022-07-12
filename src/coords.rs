use serde::{Deserialize, Serialize};
use std::convert::From;
use std::ops::{Add, Mul, Sub};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    pub fn new(x: i32, y: i32) -> Coord {
        Coord { x, y }
    }

    pub fn from_slice(coords: &[(i32, i32)]) -> Vec<Coord> {
        let mut coord_vec = vec![];
        for coord in coords.iter() {
            coord_vec.push(Coord::new(coord.0, coord.1));
        }
        coord_vec
    }
}

impl From<(i32, i32)> for Coord {
    fn from(c: (i32, i32)) -> Coord {
        Coord { x: c.0, y: c.1 }
    }
}

impl<'a, 'b> Add<&'a Coord> for &'b Coord {
    type Output = Coord;
    fn add(self, rhs: &'a Coord) -> Self::Output {
        Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<'a> Add<&'a Coord> for Coord {
    type Output = Coord;
    fn add(self, rhs: &'a Coord) -> Self::Output {
        Coord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<'a, 'b> Sub<&'a Coord> for &'b Coord {
    type Output = Coord;
    fn sub(self, rhs: &'a Coord) -> Self::Output {
        Coord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<'a> Sub<&'a Coord> for Coord {
    type Output = Coord;
    fn sub(self, rhs: &'a Coord) -> Self::Output {
        Coord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<'a> Mul<i32> for &'a Coord {
    type Output = Coord;
    fn mul(self, rhs: i32) -> Self::Output {
        Coord {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
