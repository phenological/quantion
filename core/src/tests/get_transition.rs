use crate::{
    ErrorKind, GetTransitionMetadataOptions, ParseIonOptions, SampleFile, ScanParam, ScanParamKind,
    TransitionQuery, get_transition, get_transition_metadata, parse_ion,
};

const SRM_FILE: &str = "tests/fixtures/srm.ion";
const ALANINE: f64 = 89.04768;
const SERINE: f64 = 90.0555;

fn open_srm_file() -> SampleFile {
    parse_ion(SRM_FILE, ParseIonOptions::default()).expect("open srm.ion")
}

fn by_transition(precursor: f64, product: f64) -> TransitionQuery {
    TransitionQuery::Transition { precursor, product }
}

#[test]
fn an_index_reads_each_stored_chromatogram() {
    let mut file = open_srm_file();
    for index in 0..2 {
        let trace = get_transition(&mut file, TransitionQuery::Index(index)).expect("read");
        assert!(!trace.x.is_empty());
        assert_eq!(trace.x.len(), trace.y.len());
        assert!(trace.x.windows(2).all(|pair| pair[0] <= pair[1]));
    }
}

#[test]
fn an_index_past_the_end_gives_an_empty_chromatogram() {
    let trace = get_transition(&mut open_srm_file(), TransitionQuery::Index(2)).expect("read");
    assert!(trace.x.is_empty());
    assert!(trace.y.is_empty());
}

#[test]
fn a_transition_reads_the_same_chromatogram_as_its_index() {
    let mut file = open_srm_file();
    let by_transition = get_transition(&mut file, by_transition(ALANINE, ALANINE)).expect("read");
    let by_index: Vec<Vec<f64>> = (0..2)
        .map(|index| {
            get_transition(&mut file, TransitionQuery::Index(index))
                .expect("read")
                .y
        })
        .collect();
    assert!(!by_transition.y.is_empty());
    assert!(by_index.contains(&by_transition.y));
}

#[test]
fn a_transition_matches_within_the_tolerance() {
    let near = get_transition(&mut open_srm_file(), by_transition(89.0526, 89.0436)).expect("read");
    let exact =
        get_transition(&mut open_srm_file(), by_transition(ALANINE, ALANINE)).expect("read");
    assert_eq!(near.y, exact.y);
}

#[test]
fn a_wrong_product_gives_an_empty_chromatogram() {
    let trace = get_transition(&mut open_srm_file(), by_transition(ALANINE, SERINE)).expect("read");
    assert!(trace.x.is_empty());
}

#[test]
fn a_precursor_below_zero_is_refused() {
    let error = get_transition(&mut open_srm_file(), by_transition(-1.0, ALANINE))
        .expect_err("negative precursor");
    assert_eq!(error.kind, ErrorKind::InvalidArgument);
    assert_eq!(error.argument.as_deref(), Some("precursor"));
}

fn get_rows(query: TransitionQuery, tags: &[&str]) -> Vec<ScanParam> {
    let options = GetTransitionMetadataOptions {
        tags: tags.iter().map(|tag| tag.to_string()).collect(),
    };
    get_transition_metadata(&mut open_srm_file(), query, options).expect("metadata")
}

fn get_values(rows: &[ScanParam]) -> Vec<(&str, &str, Option<&str>)> {
    rows.iter()
        .map(|row| (row.path.as_str(), row.name.as_str(), row.value.as_deref()))
        .collect()
}

#[test]
fn an_index_reads_every_row_of_its_chromatogram() {
    let rows = get_rows(TransitionQuery::Index(0), &[]);
    assert_eq!(rows.len(), 11);
    assert_eq!(
        get_values(&rows[..5]),
        vec![
            ("chromatogram", "index", Some("0")),
            (
                "chromatogram",
                "id",
                Some("SRM SIC Q1=89.04768 Q3=89.04768")
            ),
            ("chromatogram", "defaultArrayLength", Some("1035")),
            (
                "chromatogram/precursor/isolationWindow",
                "isolation window target m/z",
                Some("89.04768")
            ),
            (
                "chromatogram/product/isolationWindow",
                "isolation window target m/z",
                Some("89.04768")
            ),
        ]
    );
    assert_eq!(rows[3].kind, ScanParamKind::CV);
    assert_eq!(rows[3].accession.as_deref(), Some("MS:1000827"));
    assert_eq!(
        rows[6].path,
        "chromatogram/binaryDataArrayList/binaryDataArray[0]"
    );
    assert_eq!(rows[6].name, "time array");
}

#[test]
fn a_transition_reads_the_same_rows_as_its_index() {
    assert_eq!(
        get_rows(by_transition(SERINE, SERINE), &[]),
        get_rows(TransitionQuery::Index(1), &[])
    );
    assert_eq!(
        get_rows(by_transition(89.0526, 89.0436), &[]),
        get_rows(TransitionQuery::Index(0), &[])
    );
}

#[test]
fn tags_keep_only_their_rows() {
    let rows = get_rows(TransitionQuery::Index(0), &["chromatogram"]);
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| row.kind == ScanParamKind::ATTRIBUTE));
    let windows = get_rows(TransitionQuery::Index(0), &["isolationWindow"]);
    assert_eq!(
        windows
            .iter()
            .map(|row| row.path.as_str())
            .collect::<Vec<_>>(),
        vec![
            "chromatogram/precursor/isolationWindow",
            "chromatogram/product/isolationWindow"
        ]
    );
    assert!(get_rows(TransitionQuery::Index(0), &["precursor"]).is_empty());
}

#[test]
fn no_matching_chromatogram_gives_no_rows() {
    assert!(get_rows(TransitionQuery::Index(2), &[]).is_empty());
    assert!(get_rows(by_transition(ALANINE, SERINE), &[]).is_empty());
}

#[test]
fn an_unknown_tag_is_refused() {
    let options = GetTransitionMetadataOptions {
        tags: vec!["scan".to_string()],
    };
    let error = get_transition_metadata(&mut open_srm_file(), TransitionQuery::Index(0), options)
        .expect_err("unknown tag");
    assert_eq!(error.kind, ErrorKind::InvalidArgument);
    assert_eq!(error.argument.as_deref(), Some("tags"));
}

#[test]
fn a_metadata_product_below_zero_is_refused() {
    let error = get_transition_metadata(
        &mut open_srm_file(),
        by_transition(ALANINE, -1.0),
        GetTransitionMetadataOptions::default(),
    )
    .expect_err("negative product");
    assert_eq!(error.kind, ErrorKind::InvalidArgument);
    assert_eq!(error.argument.as_deref(), Some("product"));
}
