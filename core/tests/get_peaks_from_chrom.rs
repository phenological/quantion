use ionic::{IonReader, ReadOptions, mzml::structs::MzML};
use quantion::{
    ChromTarget,
    utilities::{find_peaks::FindPeaksOptions, get_peaks_from_chrom::get_peaks_from_chrom},
};

fn read_transitions() -> MzML {
    let bytes = std::fs::read("tests/fixtures/srm.ion").expect("read srm.ion");
    IonReader::from_bytes(&bytes, &ReadOptions::default())
        .expect("open srm.ion")
        .to_mzml()
        .expect("decode srm.ion")
}

fn get_target(id: &str, precursor: f64, product: f64) -> ChromTarget {
    ChromTarget {
        id: id.to_string(),
        precursor,
        product,
        rt: 2.385,
        range: 0.1,
    }
}

#[test]
fn a_target_finds_the_chromatogram_with_its_precursor_and_product() {
    let targets = [get_target("alanine", 89.04768, 89.04768)];
    let rows = get_peaks_from_chrom(&read_transitions(), &targets, FindPeaksOptions::default())
        .expect("chromatograms");
    assert_eq!(rows[0].id, "alanine");
    assert!(rows[0].intensity > 0.0);
    assert!((rows[0].peak_rt - 2.385).abs() < 0.05);
}

#[test]
fn a_target_matches_within_the_tolerance() {
    let targets = [get_target("alanine", 89.0526, 89.0436)];
    let rows = get_peaks_from_chrom(&read_transitions(), &targets, FindPeaksOptions::default())
        .expect("chromatograms");
    assert!(rows[0].intensity > 0.0);
}

#[test]
fn a_wrong_product_finds_nothing() {
    let targets = [get_target("alanine", 89.04768, 90.0555)];
    let rows = get_peaks_from_chrom(&read_transitions(), &targets, FindPeaksOptions::default())
        .expect("chromatograms");
    assert_eq!(rows[0].id, "alanine");
    assert_eq!(rows[0].intensity, 0.0);
}
