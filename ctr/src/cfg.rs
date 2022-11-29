use crate::{
    ipc::{Command, PermissionBuffer},
    res::CtrResult,
    service_session::{create_session_manager, session},
    srv::get_service_handle_direct,
    utils::convert::bytes_to_utf16le_string,
};
use alloc::string::String;
use core::convert::TryInto;
use no_std_io::{EndianRead, EndianWrite};

create_session_manager!({
    get_service_handle_direct("cfg:i")
        .or_else(|_| get_service_handle_direct("cfg:s"))
        .or_else(|_| get_service_handle_direct("cfg:u"))?
});

#[derive(EndianRead, EndianWrite)]
struct LocalFriendCodeSeedIn {
    out_size: u32,
    out: PermissionBuffer,
}

fn get_local_friend_code_seed_data_impl() -> CtrResult<[u8; 0x110]> {
    let mut out: [u8; 0x110] = [0; 0x110];

    let input = LocalFriendCodeSeedIn {
        out_size: out.len().try_into()?,
        out: PermissionBuffer::new_write(&mut out),
    };

    Command::new(0x4040042, input).send(get_handle())?;

    Ok(out)
}

#[derive(EndianRead, EndianWrite)]
struct ConfigInfoBlk2In {
    out_size: u32,
    block_id: u32,
    out: PermissionBuffer,
}

pub fn get_config_info_blk2(out: &mut [u8], block_id: u32) -> CtrResult {
    let out_size = out.len().try_into()?;

    let input = ConfigInfoBlk2In {
        out_size,
        block_id,
        out: PermissionBuffer::new_write(out),
    };

    Command::new(0x10082, input).send(get_handle())?;

    Ok(())
}

pub fn get_local_friend_code_seed_data() -> CtrResult<[u8; 0x110]> {
    session!(cfg);
    get_local_friend_code_seed_data_impl()
}

pub fn get_console_username() -> CtrResult<String> {
    session!(cfg);

    let mut username_bytes: [u8; 30] = [0; 30];
    // Remove two for the utf16 null terminator
    get_config_info_blk2(&mut username_bytes[0..28], 0xa0000)?;
    bytes_to_utf16le_string(&username_bytes)
}
