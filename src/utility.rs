use std::collections::BTreeSet;
use std::fmt;

pub fn is_white(color: (u32, u32, u32)) -> bool {
    (color.0 + color.1 + color.2) / 3u32 >= 240
}

pub fn is_red(color: (u32, u32, u32)) -> bool {
    (color.0 as f32).powf(2.0) / (color.1 + color.2) as f32 >= 310.0
}

pub fn adjacent_bucket(a: u32, b: u32) -> bool {
    (a as i32 - b as i32).abs() == 1
}

pub fn bmp_pixel(image: &[u8], width: u32, i: u32, j: u32) -> (u32, u32, u32) {
    (
        image[(3 * (j * width + i) + 2) as usize] as u32,
        image[(3 * (j * width + i) + 1) as usize] as u32,
        image[(3 * (j * width + i) + 0) as usize] as u32,
    )
}

/// Defines the different types of color patterns for dot searching
#[derive(Debug, Clone, Copy)]
pub enum ColorPattern {
    RedWhiteRed,
    Red,
    White,
}

impl ColorPattern {
    pub fn incr(&mut self) {
        match self.clone() {
            ColorPattern::RedWhiteRed => *self = ColorPattern::Red,
            ColorPattern::Red => *self = ColorPattern::White,
            ColorPattern::White => *self = ColorPattern::RedWhiteRed,
        }
    }
}

/// Defines the different typoes of colors a bucket can have
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SimpleColor {
    White,
    Red,
    Other,
}

impl SimpleColor {
    pub fn from_color(color: (u32, u32, u32)) -> SimpleColor {
        if is_white(color) {
            SimpleColor::White
        } else if is_red(color) {
            SimpleColor::Red
        } else {
            SimpleColor::Other
        }
    }
    pub fn max(self, other: SimpleColor) -> SimpleColor {
        use SimpleColor::*;
        if self == other {
            self
        } else {
            match self {
                White => White,
                Red => match other {
                    White => White,
                    _ => Red,
                },
                Other => other,
            }
        }
    }
}

impl fmt::Display for SimpleColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use SimpleColor::*;
        match *self {
            White => write!(f, "white"),
            Red => write!(f, "red  "),
            Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bucket {
    pub keys: BTreeSet<u32>,
    pub points: BTreeSet<u32>,
    pub simple_color: SimpleColor,
}

impl Bucket {
    pub fn new() -> Bucket {
        Bucket {
            keys: BTreeSet::new(),
            points: BTreeSet::new(),
            simple_color: SimpleColor::Other,
        }
    }
    pub fn main_key(&self) -> u32 {
        *self.keys.iter().next().unwrap()
    }
    pub fn insert(&mut self, key: u32, point: u32, simple_color: SimpleColor) {
        self.keys.insert(key);
        self.points.insert(point);
        self.simple_color = self.simple_color.max(simple_color);
    }
    pub fn merge(mut self, other: &mut Bucket) -> Bucket {
        self.keys.append(&mut other.keys);
        self.points.append(&mut other.points);
        Bucket {
            keys: self.keys,
            points: self.points,
            simple_color: self.simple_color,
        }
    }
    pub fn adjacent(&self, other: &Bucket) -> bool {
        for i in &self.keys {
            for j in &other.keys {
                if adjacent_bucket(*i, *j) && self.simple_color == other.simple_color {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Lookup {
    Exact(f32),
    Mid(i32, i32),
    Blank,
}

#[derive(Debug, Clone)]
pub struct LookupTable {
    v: Vec<Lookup>,
}

impl LookupTable {
    pub fn new(n: usize) -> LookupTable {
        LookupTable {
            v: vec![Lookup::Blank; n],
        }
    }
    pub fn add_exact(&mut self, i: usize, dist: f32) {
        self.v[i as usize] = Lookup::Exact(dist);
    }
    pub fn exact(&self, i: usize) -> f32 {
        if let Lookup::Exact(dist) = self.v[i] {
            dist
        } else {
            panic!("Called exact() with non-exact index");
        }
    }
    pub fn fill(&mut self) {
        use Lookup::*;
        let size = self.v.len();
        for i in 0..size {
            if let Exact(_) = self.v[i] {
                for j in (i + 1)..size {
                    match self.v[j] {
                        Exact(_) => break,
                        Blank => self.v[j] = Mid(i as i32, -1),
                        _ => (),
                    }
                }
            }
        }
        for i in (1..size).rev() {
            if let Exact(_) = self.v[i] {
                for j in (0..(i - 1)).rev() {
                    match self.v[j].clone() {
                        Exact(_) => break,
                        Mid(..) => {
                            if let Mid(ref _a, ref mut b) = self.v[j] {
                                *b = i as i32
                            }
                        }
                        Blank => self.v[j] = Mid(-1, i as i32),
                    }
                }
            }
        }
    }
    pub fn dist(&self, i: usize) -> f32 {
        use Lookup::*;
        match self.v[i] {
            Exact(dist) => dist,
            Mid(a, b) => {
                if a < 0 || b < 0 {
                    -1.0
                } else {
                    ((i as i32 - a) as f32 * self.exact(b as usize)
                        + (b - i as i32) as f32 * self.exact(a as usize))
                        / (b - a) as f32
                }
            }
            Blank => panic!("Found a blank on call to lookup()"),
        }
    }
}
