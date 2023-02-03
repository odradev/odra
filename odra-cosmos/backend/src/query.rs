use cosmwasm_std::{ContractResult, Env, QueryResponse};

use crate::{
    runtime::{Runtime, RT},
    utils::{self, consume_region, Region}
};

pub fn query<E: ToString>(
    query_fn: &dyn Fn(&[u8]) -> Result<QueryResponse, E>,
    env_ptr: u32,
    msg_ptr: u32
) -> u32 {
    #[cfg(feature = "abort")]
    install_panic_handler();
    let env: Vec<u8> = unsafe { consume_region(env_ptr as *mut Region) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr as *mut Region) };
    let env: Env = match cosmwasm_std::from_slice(&env) {
        Ok(val) => val,
        Err(err) => {
            return utils::err_to_u32::<QueryResponse>(err);
        }
    };

    RT.with(|rt_ref| rt_ref.replace(Runtime::query(env)));
    let result: ContractResult<QueryResponse> = query_fn(&msg).into();
    RT.with(|rt_ref| rt_ref.replace(Runtime::default()));

    let v = serde_json_wasm::to_vec(&result).unwrap();
    crate::utils::release_buffer(v) as u32
}
