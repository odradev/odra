use std::mem;

use cosmwasm_std::{ContractResult, StdError};

#[repr(C)]
pub struct Region {
    /// The beginning of the region expressed as bytes from the beginning of the linear memory
    pub offset: u32,
    /// The number of bytes available in this region
    pub capacity: u32,
    /// The number of bytes used in this region
    pub length: u32
}

pub unsafe fn consume_region(ptr: *mut Region) -> Vec<u8> {
    assert!(!ptr.is_null(), "Region pointer is null");
    let region = Box::from_raw(ptr);

    let region_start = region.offset as *mut u8;
    // This case is explicitly disallowed by Vec
    // "The pointer will never be null, so this type is null-pointer-optimized."
    assert!(!region_start.is_null(), "Region starts at null pointer");

    Vec::from_raw_parts(
        region_start,
        region.length as usize,
        region.capacity as usize
    )
}

pub fn release_buffer(buffer: Vec<u8>) -> *mut Region {
    let region = build_region(&buffer);
    mem::forget(buffer);
    Box::into_raw(region)
}

pub fn build_region(data: &[u8]) -> Box<Region> {
    let data_ptr = data.as_ptr() as usize;
    build_region_from_components(
        u32::try_from(data_ptr).expect("pointer doesn't fit in u32"),
        u32::try_from(data.len()).expect("length doesn't fit in u32"),
        u32::try_from(data.len()).expect("length doesn't fit in u32")
    )
}

fn build_region_from_components(offset: u32, capacity: u32, length: u32) -> Box<Region> {
    Box::new(Region {
        offset,
        capacity,
        length
    })
}

pub fn err_to_u32<C: serde::Serialize>(err: StdError) -> u32 {
    let result: ContractResult<C> = ContractResult::Err(err.to_string());
    let v = serde_json_wasm::to_vec(&result).unwrap();
    return crate::utils::release_buffer(v) as u32;
}
