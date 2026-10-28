use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct DataXY {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

impl DataXY {
    pub fn empty() -> DataXY {
        Self {
            x: Vec::new(),
            y: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct FromTo {
    pub from: f64,
    pub to: f64,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Peak {
    pub from: f64,
    pub to: f64,
    pub rt: f64,
    pub integral: f64,
    pub intensity: f64,
    pub n_points: usize,
    pub noise: f64,
    pub r2: Option<f64>,
}

impl Default for Peak {
    fn default() -> Self {
        Self {
            from: 0.0,
            to: 0.0,
            rt: 0.0,
            integral: 0.0,
            intensity: 0.0,
            r2: None,
            n_points: 0,
            noise: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Roi {
    pub rt: f64,
    pub range: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Target {
    pub id: String,
    pub mz: f64,
    pub rt: f64,
    pub range: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChromTarget {
    pub id: String,
    pub precursor: f64,
    pub product: f64,
    pub rt: f64,
    pub range: f64,
}
