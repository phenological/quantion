use crate::utilities::{
    cheminfo::sgg::{SggOptions, sgg},
    closest_index,
    structs::DataXY,
};

#[derive(Clone, Copy, Debug)]
pub struct Boundary {
    pub index: Option<usize>,
    pub value: Option<f64>,
}

#[derive(Clone, Copy, Debug)]
pub struct Boundaries {
    pub from: Boundary,
    pub to: Boundary,
}

#[derive(Clone, Copy, Debug)]
pub struct BoundariesOptions {
    pub min_slope_step: f64,
    pub smooth_window: usize,
    pub smooth_polynomial: usize,
}

impl Default for BoundariesOptions {
    fn default() -> Self {
        Self {
            min_slope_step: 1e-5,
            smooth_window: 7,
            smooth_polynomial: 3,
        }
    }
}

pub fn get_boundaries(
    data: &DataXY,
    peak_x: f64,
    options: Option<BoundariesOptions>,
) -> Boundaries {
    let n = data.x.len();
    if n < 2 || n != data.y.len() {
        return Boundaries {
            from: Boundary {
                index: None,
                value: None,
            },
            to: Boundary {
                index: None,
                value: None,
            },
        };
    }

    let options = options.unwrap_or_default();
    let apex_index = closest_index(&data.x, peak_x);
    let lowest_value = min_value(&data.y);

    find_edges(data, apex_index, &options, lowest_value)
}

fn min_value(values: &[f64]) -> f64 {
    values.iter().copied().fold(f64::INFINITY, f64::min)
}

fn usable_window(requested: usize, length: usize) -> Option<usize> {
    if length < 5 {
        return None;
    }
    let mut window = requested.min(length);
    if window.is_multiple_of(2) {
        window -= 1;
    }
    (window >= 5).then_some(window)
}

fn find_edges(
    data: &DataXY,
    apex_index: usize,
    options: &BoundariesOptions,
    lowest_value: f64,
) -> Boundaries {
    let smoothed = match usable_window(options.smooth_window, data.y.len()) {
        Some(window) => sgg(
            &data.y,
            &data.x,
            SggOptions {
                window_size: window,
                derivative: 0,
                polynomial: options.smooth_polynomial,
            },
        ),
        None => data.y.to_vec(),
    };

    let floor = lowest_value + options.min_slope_step;

    Boundaries {
        from: find_edge(&data.x, &smoothed, apex_index, -1, floor),
        to: find_edge(&data.x, &smoothed, apex_index, 1, floor),
    }
}

fn find_edge(
    x: &[f64],
    smoothed: &[f64],
    apex_index: usize,
    direction: isize,
    floor: f64,
) -> Boundary {
    let length = smoothed.len() as isize;
    let mut current = apex_index as isize;
    let mut lowest = smoothed[apex_index];
    let mut lowest_index = apex_index;

    while current + direction >= 0 && current + direction < length {
        let next = (current + direction) as usize;
        if smoothed[next] <= floor {
            return boundary_at(x, next);
        }
        if smoothed[next] < lowest {
            lowest = smoothed[next];
            lowest_index = next;
        }
        current = next as isize;
    }

    boundary_at(x, lowest_index)
}

fn boundary_at(x: &[f64], index: usize) -> Boundary {
    Boundary {
        index: Some(index),
        value: Some(x[index]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(from: f64, to: f64, points: usize) -> Vec<f64> {
        (0..points)
            .map(|index| from + (to - from) * index as f64 / (points - 1) as f64)
            .collect()
    }

    fn bell(x: f64, center: f64, height: f64, fwhm: f64) -> f64 {
        let sigma = fwhm / 2.354_820_045;
        height * (-0.5 * ((x - center) / sigma).powi(2)).exp()
    }

    fn wobble(index: usize) -> f64 {
        let value = (index as f64 * 12.9898).sin() * 43758.547;
        2.0 * (value - value.floor()) - 1.0
    }

    #[test]
    fn brackets_the_apex() {
        let x = grid(4.0, 6.0, 400);
        let y: Vec<f64> = x.iter().map(|&v| 0.05 + bell(v, 5.0, 1.0, 0.2)).collect();
        let data = DataXY { x, y };

        let edges = get_boundaries(&data, 5.0, Some(BoundariesOptions::default()));
        let from = edges.from.value.unwrap();
        let to = edges.to.value.unwrap();

        assert!(
            from < 5.0 && to > 5.0,
            "bracket [{from}, {to}] must contain the apex"
        );
        assert!(
            from < 4.9 && to > 5.1,
            "bracket [{from}, {to}] must reach past the flanks"
        );
    }

    #[test]
    fn stops_at_the_valley_between_two_peaks() {
        let x = grid(4.0, 6.5, 500);
        let y: Vec<f64> = x
            .iter()
            .map(|&v| 0.05 + bell(v, 5.0, 1.0, 0.2) + bell(v, 5.35, 0.8, 0.2))
            .collect();
        let cut = x.partition_point(|&v| v <= 5.35);
        let data = DataXY {
            x: x[..cut].to_vec(),
            y: y[..cut].to_vec(),
        };

        let to = get_boundaries(&data, 5.0, Some(BoundariesOptions::default()))
            .to
            .value
            .unwrap();

        assert!(
            to > 5.0 && to < 5.35,
            "right edge {to} must land in the valley, not the next peak"
        );
    }

    #[test]
    fn window_is_stable_under_noise() {
        let x = grid(4.0, 6.5, 500);
        let clean: Vec<f64> = x
            .iter()
            .map(|&v| 0.05 + bell(v, 5.0, 1.0, 0.2) + bell(v, 5.35, 0.8, 0.2))
            .collect();
        let noisy: Vec<f64> = clean
            .iter()
            .enumerate()
            .map(|(index, &value)| value + 0.02 * wobble(index))
            .collect();

        let cut = x.partition_point(|&v| v <= 5.35);
        let x = x[..cut].to_vec();
        let clean_to = get_boundaries(
            &DataXY {
                x: x.clone(),
                y: clean[..cut].to_vec(),
            },
            5.0,
            Some(BoundariesOptions::default()),
        )
        .to
        .value
        .unwrap();
        let noisy_to = get_boundaries(
            &DataXY {
                x,
                y: noisy[..cut].to_vec(),
            },
            5.0,
            Some(BoundariesOptions::default()),
        )
        .to
        .value
        .unwrap();

        assert!(
            (clean_to - noisy_to).abs() <= 0.05,
            "right edge moved {clean_to} -> {noisy_to} under noise"
        );
    }

    #[test]
    fn keeps_left_edge_when_right_side_is_truncated() {
        let full_x = grid(4.0, 6.0, 400);
        let full_y: Vec<f64> = full_x
            .iter()
            .map(|&v| 0.05 + bell(v, 5.0, 1.0, 0.2))
            .collect();
        let full_from = get_boundaries(
            &DataXY {
                x: full_x.clone(),
                y: full_y.clone(),
            },
            5.0,
            Some(BoundariesOptions::default()),
        )
        .from
        .value
        .unwrap();

        let cut = full_x.partition_point(|&v| v <= 5.25);
        let cut_x = full_x[..cut].to_vec();
        let cut_y = full_y[..cut].to_vec();
        let cut_edges = get_boundaries(
            &DataXY { x: cut_x, y: cut_y },
            5.0,
            Some(BoundariesOptions::default()),
        );
        let cut_from = cut_edges.from.value.unwrap();
        let cut_to = cut_edges.to.value.unwrap();

        assert!(
            (full_from - cut_from).abs() <= 0.02,
            "left edge moved {full_from} -> {cut_from} after truncation"
        );
        assert!(
            cut_to >= 5.2,
            "right edge {cut_to} should follow the data to its truncated end"
        );
    }

    #[test]
    fn short_input_does_not_panic() {
        let x = grid(0.0, 1.0, 4);
        let y = vec![0.0, 1.0, 0.5, 0.2];
        let edges = get_boundaries(&DataXY { x, y }, 0.3, Some(BoundariesOptions::default()));
        assert!(edges.from.index.is_some() && edges.to.index.is_some());
    }
}
