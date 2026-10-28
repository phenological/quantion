use core::ffi::c_int;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    ptr, slice,
};

use super::{
    Buf, ERR_ENCODE, ERR_INVALID_ARGS, ERR_PANIC, FileSource, OK, ParsedFile, clear_buf, f64_to_u8,
    fast_error_to_code, file_is_broken, mark_file_broken,
    model::{ERR_MODEL_NOT_READ, ModelHandle},
    pack_byte_ranges, write_buf,
};
use crate::utilities::{
    bridge::{
        Column, QUANTION_PAYLOAD_WARP_TARGETS, QUANTION_SECTION_WARP_TARGET_ID_BYTES,
        QUANTION_SECTION_WARP_TARGET_ID_STARTS, QUANTION_SECTION_WARP_TARGET_MZ,
        QUANTION_SECTION_WARP_TARGET_RT, build_record_bridge,
    },
    calculate_eic::{EicReader, FastError},
    structs::Target,
    warp_model::{
        WarpError, WarpModel, check_target_lists, get_median_targets, load_warp,
        plan_predict_rts_ranges,
    },
};

const ERR_TARGET_MISSING: c_int = 8;
const ERR_SCANS_NOT_COVERED: c_int = 9;
const ERR_NO_BACKGROUND: c_int = 10;
const ERR_PREDICT_FAILED: c_int = 11;

pub struct WarpModelHandle {
    warp: WarpModel,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warp_model_load(
    model: *const ModelHandle,
    out: *mut *mut WarpModelHandle,
    message: *mut Buf,
) -> c_int {
    if model.is_null() || out.is_null() || message.is_null() {
        return ERR_INVALID_ARGS;
    }
    unsafe { *out = ptr::null_mut() };
    clear_buf(message);
    let model = unsafe { &(*model).model };
    match catch_unwind(AssertUnwindSafe(|| load_warp(model))) {
        Ok(Ok(warp)) => {
            unsafe { *out = Box::into_raw(Box::new(WarpModelHandle { warp })) };
            OK
        }
        Ok(Err(error)) => write_error(message, error),
        Err(_) => ERR_PANIC,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warp_model_free(warp: *mut WarpModelHandle) {
    if !warp.is_null() {
        drop(unsafe { Box::from_raw(warp) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn predict_rts_from_file(
    warp: *const WarpModelHandle,
    h: *const ParsedFile,
    mzs_ptr: *const f64,
    rts_ptr: *const f64,
    n: usize,
    out: *mut Buf,
) -> c_int {
    if warp.is_null() || h.is_null() || mzs_ptr.is_null() || rts_ptr.is_null() || out.is_null() {
        return ERR_INVALID_ARGS;
    }
    if file_is_broken(h) {
        return ERR_PANIC;
    }
    clear_buf(out);

    match catch_unwind(AssertUnwindSafe(|| -> Result<(), c_int> {
        let file = unsafe { &mut *(h as *mut ParsedFile) };
        let targets = get_targets(mzs_ptr, rts_ptr, n);
        let mut reader = match file.source_mut() {
            FileSource::Full(mzml) => EicReader::Mzml(mzml.as_mut()),
            FileSource::Lazy(ion) => EicReader::Ion(ion.as_mut()),
            FileSource::Remote(ion) => EicReader::Ion(ion.as_mut()),
        };

        let predicted = unsafe { &(*warp).warp }
            .predict_rts(&mut reader, &targets)
            .map_err(|error| write_error(out, error))?;
        let values: Vec<f64> = predicted
            .iter()
            .flat_map(|prediction| [prediction.rt, prediction.uncertainty])
            .collect();
        write_buf(out, f64_to_u8(&values));
        Ok(())
    })) {
        Ok(Ok(())) => OK,
        Ok(Err(code)) => code,
        Err(_) => {
            mark_file_broken(h);
            ERR_PANIC
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn plan_predict_rts(
    warp: *const WarpModelHandle,
    h: *mut ParsedFile,
    mzs_ptr: *const f64,
    rts_ptr: *const f64,
    n: usize,
    out: *mut Buf,
) -> c_int {
    if warp.is_null() || h.is_null() || mzs_ptr.is_null() || rts_ptr.is_null() || out.is_null() {
        return ERR_INVALID_ARGS;
    }
    if file_is_broken(h) {
        return ERR_PANIC;
    }
    clear_buf(out);

    match catch_unwind(AssertUnwindSafe(|| -> Result<(), c_int> {
        let file = unsafe { &mut *h };
        let targets = get_targets(mzs_ptr, rts_ptr, n);
        let warp = unsafe { &(*warp).warp };
        let ranges = match file.source_mut() {
            FileSource::Lazy(ion) => plan_predict_rts_ranges(warp, ion.as_mut(), &targets),
            FileSource::Remote(ion) => plan_predict_rts_ranges(warp, ion.as_mut(), &targets),
            FileSource::Full(_) => Err(WarpError::FileNotRead(FastError::UnsupportedBackend)),
        }
        .map_err(|error| write_error(out, error))?;
        write_buf(
            out,
            pack_byte_ranges(ranges.iter().map(|range| (range.offset, range.length))),
        );
        Ok(())
    })) {
        Ok(Ok(())) => OK,
        Ok(Err(code)) => code,
        Err(_) => {
            mark_file_broken(h);
            ERR_PANIC
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn calibrate_rts(
    warp: *const WarpModelHandle,
    files: *const *const ParsedFile,
    file_count: usize,
    list_lengths: *const usize,
    ids_off: *const u32,
    ids_len: *const u32,
    ids_buf: *const u8,
    ids_buf_len: usize,
    mzs_ptr: *const f64,
    rts_ptr: *const f64,
    target_count: usize,
    out: *mut Buf,
) -> c_int {
    if warp.is_null() || out.is_null() {
        return ERR_INVALID_ARGS;
    }
    if file_count > 0 && (files.is_null() || list_lengths.is_null()) {
        return ERR_INVALID_ARGS;
    }
    if target_count > 0
        && (ids_off.is_null() || ids_len.is_null() || mzs_ptr.is_null() || rts_ptr.is_null())
    {
        return ERR_INVALID_ARGS;
    }
    clear_buf(out);
    let files = get_slice(files, file_count);
    let lengths = get_slice(list_lengths, file_count);
    if lengths.iter().sum::<usize>() != target_count
        || files
            .iter()
            .any(|file| file.is_null() || file_is_broken(*file))
    {
        return ERR_INVALID_ARGS;
    }

    let warp = unsafe { &(*warp).warp };
    let mut targets = get_targets(mzs_ptr, rts_ptr, target_count);
    match get_ids(ids_off, ids_len, ids_buf, ids_buf_len, target_count) {
        Some(ids) => targets
            .iter_mut()
            .zip(ids)
            .for_each(|(target, id)| target.id = id),
        None => return ERR_INVALID_ARGS,
    }
    let target_lists = split_targets(targets, lengths);
    if let Err(error) = check_target_lists(files.len(), &target_lists) {
        return write_error(out, error);
    }

    let mut calibrated_lists = Vec::with_capacity(files.len());
    for (file, targets) in files.iter().zip(&target_lists) {
        match catch_unwind(AssertUnwindSafe(|| {
            let parsed = unsafe { &mut *(*file as *mut ParsedFile) };
            let mut reader = match parsed.source_mut() {
                FileSource::Full(mzml) => EicReader::Mzml(mzml.as_mut()),
                FileSource::Lazy(ion) => EicReader::Ion(ion.as_mut()),
                FileSource::Remote(ion) => EicReader::Ion(ion.as_mut()),
            };
            warp.calibrate_sample(&mut reader, targets)
        })) {
            Ok(Ok(calibrated)) => calibrated_lists.push(calibrated),
            Ok(Err(error)) => return write_error(out, error),
            Err(_) => {
                mark_file_broken(*file);
                return ERR_PANIC;
            }
        }
    }

    match build_targets_bridge(&get_median_targets(&calibrated_lists)) {
        Some(bytes) => {
            write_buf(out, bytes);
            OK
        }
        None => ERR_ENCODE,
    }
}

fn split_targets(targets: Vec<Target>, lengths: &[usize]) -> Vec<Vec<Target>> {
    let mut rest = targets.into_iter();
    lengths
        .iter()
        .map(|length| rest.by_ref().take(*length).collect())
        .collect()
}

fn get_slice<'a, T>(values: *const T, count: usize) -> &'a [T] {
    if count == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(values, count) }
    }
}

fn build_targets_bridge(targets: &[Target]) -> Option<Box<[u8]>> {
    let ids: Vec<&str> = targets.iter().map(|target| target.id.as_str()).collect();
    let mz: Vec<f64> = targets.iter().map(|target| target.mz).collect();
    let rt: Vec<f64> = targets.iter().map(|target| target.rt).collect();
    build_record_bridge(
        QUANTION_PAYLOAD_WARP_TARGETS,
        targets.len() as u64,
        &[
            Column::Text {
                starts_id: QUANTION_SECTION_WARP_TARGET_ID_STARTS,
                bytes_id: QUANTION_SECTION_WARP_TARGET_ID_BYTES,
                values: &ids,
            },
            Column::Numbers {
                id: QUANTION_SECTION_WARP_TARGET_MZ,
                values: &mz,
            },
            Column::Numbers {
                id: QUANTION_SECTION_WARP_TARGET_RT,
                values: &rt,
            },
        ],
    )
}

fn get_targets(mzs_ptr: *const f64, rts_ptr: *const f64, n: usize) -> Vec<Target> {
    if n == 0 {
        return Vec::new();
    }
    let mzs = unsafe { slice::from_raw_parts(mzs_ptr, n) };
    let rts = unsafe { slice::from_raw_parts(rts_ptr, n) };
    mzs.iter()
        .zip(rts)
        .map(|(mz, rt)| Target {
            id: String::new(),
            mz: *mz,
            rt: *rt,
            range: 0.0,
        })
        .collect()
}

fn get_ids(
    ids_off: *const u32,
    ids_len: *const u32,
    ids_buf: *const u8,
    ids_buf_len: usize,
    n: usize,
) -> Option<Vec<String>> {
    let offsets = get_slice(ids_off, n);
    let lengths = get_slice(ids_len, n);
    let bytes = if ids_buf.is_null() {
        &[][..]
    } else {
        get_slice(ids_buf, ids_buf_len)
    };
    offsets
        .iter()
        .zip(lengths)
        .map(|(offset, length)| {
            let start = *offset as usize;
            let id = bytes.get(start..start.checked_add(*length as usize)?)?;
            std::str::from_utf8(id).ok().map(str::to_string)
        })
        .collect()
}

fn write_error(message: *mut Buf, error: WarpError) -> c_int {
    write_buf(message, error.to_string().into_bytes().into_boxed_slice());
    get_error_code(error)
}

fn get_error_code(error: WarpError) -> c_int {
    match error {
        WarpError::WrongInputs | WarpError::ValueMissing(_) => ERR_MODEL_NOT_READ,
        WarpError::TargetMissing(_)
        | WarpError::ListsDiffer { .. }
        | WarpError::IdMissing { .. } => ERR_TARGET_MISSING,
        WarpError::ScansNotCovered { .. } => ERR_SCANS_NOT_COVERED,
        WarpError::NoBackground => ERR_NO_BACKGROUND,
        WarpError::FileNotRead(error) => fast_error_to_code(error),
        WarpError::PredictFailed(_) => ERR_PREDICT_FAILED,
    }
}
