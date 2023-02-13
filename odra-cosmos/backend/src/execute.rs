use cosmwasm_std::{ContractResult, CustomMsg, Env, MessageInfo, Response};

use crate::{
    runtime::{Runtime, RT},
    utils::{self, consume_region, Region}
};

pub fn execute<C, E>(
    exe_fn: &dyn Fn(&[u8]) -> Result<Response<C>, E>,
    env_ptr: u32,
    info_ptr: u32,
    msg_ptr: u32
) -> u32
where
    C: CustomMsg,
    E: ToString
{
    #[cfg(target_arch = "wasm32")]
    utils::install_panic_handler();

    let env: Vec<u8> = unsafe { consume_region(env_ptr as *mut Region) };
    let info: Vec<u8> = unsafe { consume_region(info_ptr as *mut Region) };
    let msg: Vec<u8> = unsafe { consume_region(msg_ptr as *mut Region) };

    let env: Env = match cosmwasm_std::from_slice(&env) {
        Ok(val) => val,
        Err(err) => {
            return utils::err_to_u32::<Response<C>>(err);
        }
    };
    let info: MessageInfo = match cosmwasm_std::from_slice(&info) {
        Ok(val) => val,
        Err(err) => {
            return utils::err_to_u32::<Response<C>>(err);
        }
    };

    RT.with(|rt_ref| rt_ref.replace(Runtime::new(env, info)));
    let result: ContractResult<Response<C>> = exe_fn(&msg).into();
    RT.with(|rt_ref| rt_ref.replace(Runtime::default()));
    let v = serde_json_wasm::to_vec(&result).unwrap();
    crate::utils::release_buffer(v) as u32
}
