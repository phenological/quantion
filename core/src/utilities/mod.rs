pub mod calculate_eic;
pub mod ion;
pub mod ion_image;
pub mod parse_ion;
pub use calculate_eic::{
    EicOptions, EicReader, FastError, ScanTime, calculate_eic, get_scan_times, plan_eic_ranges,
    read_mz_window,
};
pub use parse_ion::{IonInput, parse_ion};

pub mod parallel;

pub mod find_features;
pub use find_features::find_features;

pub mod find_masses;

pub mod bridge;

pub mod find_noise_level;
pub use find_noise_level::{find_noise_level, find_noise_level_san_plot};

pub mod find_peaks;

pub(crate) mod resolve_apexes;

pub mod shape_filter;

pub mod fit_peak;

pub mod functions;

pub mod calculate_baseline;
pub use calculate_baseline::calculate_baseline;

pub mod get_boundaries;

pub mod get_peak;

pub mod find_transitions;

pub mod find_targets;

pub mod load_model;

pub mod warp_model;

pub mod scan_for_peaks;

pub mod structs;

pub mod cheminfo;

pub mod math;
pub use math::closest_index;

#[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
pub mod match_features;
pub mod mz_estimator;
#[cfg(test)]
mod tests;
