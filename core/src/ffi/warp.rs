use core::ffi::c_int;
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    ptr, slice,
};

use super::{
    Buf, ERR_INVALID_ARGS, ERR_PANIC, FileSource, OK, ParsedFile, clear_buf, f64_to_u8,
    fast_error_to_code, file_is_broken, mark_file_broken, pack_byte_ranges, write_buf,
};
use crate::utilities::{
    calculate_eic::{EicReader, FastError},
    warp_model::{
        ModelInput, Target, WarpError, WarpModel, load_warp_model, plan_predict_rts_ranges,
        predict_rts,
    },
};

const ERR_MODEL_NOT_READ: c_int = 7;
const ERR_TARGET_MISSING: c_int = 8;
const ERR_SCANS_NOT_COVERED: c_int = 9;
const ERR_NO_BACKGROUND: c_int = 10;
const ERR_PREDICT_FAILED: c_int = 11;

pub struct WarpModelHandle {
    warp: WarpModel,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warp_model_load_path(
    path_ptr: *const u8,
    path_len: usize,
    out: *mut *mut WarpModelHandle,
    message: *mut Buf,
) -> c_int {
    if path_ptr.is_null() || path_len == 0 || out.is_null() || message.is_null() {
        return ERR_INVALID_ARGS;
    }
    let path_bytes = unsafe { slice::from_raw_parts(path_ptr, path_len) };
    let Ok(path) = std::str::from_utf8(path_bytes) else {
        return ERR_INVALID_ARGS;
    };
    write_warp_model(out, message, ModelInput::from(std::path::Path::new(path)))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn warp_model_load_bytes(
    model_ptr: *const u8,
    model_len: usize,
    out: *mut *mut WarpModelHandle,
    message: *mut Buf,
) -> c_int {
    if model_ptr.is_null() || model_len == 0 || out.is_null() || message.is_null() {
        return ERR_INVALID_ARGS;
    }
    let bytes = unsafe { slice::from_raw_parts(model_ptr, model_len) };
    write_warp_model(out, message, ModelInput::from(bytes))
}

fn write_warp_model(out: *mut *mut WarpModelHandle, message: *mut Buf, input: ModelInput) -> c_int {
    unsafe { *out = ptr::null_mut() };
    clear_buf(message);
    match catch_unwind(AssertUnwindSafe(|| load_warp_model(input))) {
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

        let predicted = predict_rts(unsafe { &(*warp).warp }, &mut reader, &targets)
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

fn get_targets(mzs_ptr: *const f64, rts_ptr: *const f64, n: usize) -> Vec<Target> {
    let mzs = unsafe { slice::from_raw_parts(mzs_ptr, n) };
    let rts = unsafe { slice::from_raw_parts(rts_ptr, n) };
    mzs.iter()
        .zip(rts)
        .map(|(mz, rt)| Target { mz: *mz, rt: *rt })
        .collect()
}

fn write_error(message: *mut Buf, error: WarpError) -> c_int {
    write_buf(message, error.to_string().into_bytes().into_boxed_slice());
    get_error_code(error)
}

fn get_error_code(error: WarpError) -> c_int {
    match error {
        WarpError::ModelNotRead(_) | WarpError::SettingMissing(_) => ERR_MODEL_NOT_READ,
        WarpError::TargetMissing(_) => ERR_TARGET_MISSING,
        WarpError::ScansNotCovered { .. } => ERR_SCANS_NOT_COVERED,
        WarpError::NoBackground => ERR_NO_BACKGROUND,
        WarpError::FileNotRead(error) => fast_error_to_code(error),
        WarpError::PredictFailed(_) => ERR_PREDICT_FAILED,
    }
}
