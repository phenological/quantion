mod ffi;
pub mod utilities;

pub use utilities::structs::{DataXY, FromTo, Peak, Roi, Target, Transition};

pub use ionic;

pub mod io {
    pub use crate::utilities::{
        ion::{ScanSource, ScanSummary, write_mzml_to_ion},
        parse_ion::{IonInput, parse_ion},
    };
    pub use ionic::{
        IonError, IonReader, IonResult, ReadOptions, WriteOptions,
        mzml::{parse_mzml, structs::MzML},
    };
}

pub mod eic {
    pub use crate::utilities::calculate_eic::{
        EicOptions, EicReader, FastError, ScanQuery, TimeUnit, calculate_eic, get_scans,
        plan_eic_ranges,
    };
}

pub mod peaks {
    pub use crate::utilities::{
        find_peaks::{ArtifactFilter, FindPeaksOptions, PeakFilter, find_peaks},
        find_targets::{FindTargetsOptions, find_targets, plan_targets_ranges},
        find_transitions::{FindTransitionsOptions, find_transitions},
        fit_peak::{PeakParameters, PeakSeed, PeakShape, draw_peak, fit_peak},
        get_peak::get_peak,
    };
}

pub mod model {
    pub use crate::utilities::load_model::{Model, ModelError, ModelInput, load_model};
}

pub mod warp {
    pub use crate::utilities::warp_model::{PredictedRt, WarpError, WarpModel, load_warp};
}

pub mod boundaries {
    pub use crate::utilities::get_boundaries::{
        Boundaries, BoundariesOptions, Boundary, get_boundaries,
    };
}

pub mod noise {
    pub use crate::utilities::find_noise_level::{find_noise_level, find_noise_level_san_plot};
}

pub mod baseline {
    pub use crate::utilities::calculate_baseline::{BaselineOptions, calculate_baseline};
}

pub mod features {
    #[cfg(not(all(target_arch = "wasm32", not(target_os = "wasi"))))]
    pub use crate::utilities::match_features::{
        MatchFeaturesOptions, match_detected_features, match_features,
    };
    pub use crate::utilities::{
        find_features::{FindFeaturesOptions, find_features},
        mz_estimator::MzEstimatorKind,
    };
}
