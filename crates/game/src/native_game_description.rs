// SPDX-License-Identifier: GPL-3.0-or-later
// PRIVATE metadata port of pristine games/wiiparty/game.cpp description;mechanically extracted,never commit/upload.
use std::ffi::{c_char,c_void};
// Public standard-layout POD from engine/include/wp/game.h,not an opaque C++ class.
#[repr(C)]pub struct Description{
pub title:*const c_char,
pub short_name:*const c_char,
pub data_directory:*const c_char,
pub nand_directory:*const c_char,
pub settings_file:*const c_char,
pub textures_directory:*const c_char,
pub notice_module:*const c_char,
pub sideways_modules:*const *const c_char,
pub sideways_module_count:usize,
pub instruction_module:*const c_char,
pub minigame_prefix:*const c_char,
}
unsafe impl Sync for Description{}
struct Sideways([*const c_char;37]);
unsafe impl Sync for Sideways{}
static SIDEWAYS:Sideways=Sideways([b"mg102\0".as_ptr().cast(),b"mg104\0".as_ptr().cast(),b"mg108\0".as_ptr().cast(),b"mg109\0".as_ptr().cast(),b"mg202\0".as_ptr().cast(),b"mg210\0".as_ptr().cast(),b"mg212\0".as_ptr().cast(),b"mg213\0".as_ptr().cast(),b"mg218\0".as_ptr().cast(),b"mg221\0".as_ptr().cast(),b"mg222\0".as_ptr().cast(),b"mg223\0".as_ptr().cast(),b"mg224\0".as_ptr().cast(),b"mg302\0".as_ptr().cast(),b"mg404\0".as_ptr().cast(),b"mg407\0".as_ptr().cast(),b"mg412\0".as_ptr().cast(),b"mg415\0".as_ptr().cast(),b"mg422\0".as_ptr().cast(),b"mg428\0".as_ptr().cast(),b"mg430\0".as_ptr().cast(),b"mg431\0".as_ptr().cast(),b"mg432\0".as_ptr().cast(),b"mg433\0".as_ptr().cast(),b"mg436\0".as_ptr().cast(),b"mg437\0".as_ptr().cast(),b"mg438\0".as_ptr().cast(),b"mg440\0".as_ptr().cast(),b"mg441\0".as_ptr().cast(),b"mg445\0".as_ptr().cast(),b"mg446\0".as_ptr().cast(),b"mg503\0".as_ptr().cast(),b"mg504\0".as_ptr().cast(),b"mg505\0".as_ptr().cast(),b"mg507\0".as_ptr().cast(),b"mg508\0".as_ptr().cast(),b"mg509\0".as_ptr().cast()]);
static DESCRIPTION:Description=Description{
title:b"Wii Party\0".as_ptr().cast(),
short_name:b"wiiparty\0".as_ptr().cast(),
data_directory:b"games/wiiparty/extracted\0".as_ptr().cast(),
nand_directory:b"games/wiiparty/nand\0".as_ptr().cast(),
settings_file:b"games/wiiparty/settings.ini\0".as_ptr().cast(),
textures_directory:b"games/wiiparty/textures\0".as_ptr().cast(),
notice_module:b"boot\0".as_ptr().cast(),
sideways_modules:SIDEWAYS.0.as_ptr(),
sideways_module_count:SIDEWAYS.0.len(),
instruction_module:b"inst\0".as_ptr().cast(),
minigame_prefix:b"mg\0".as_ptr().cast(),
};
// port of games/wiiparty/game.cpp description via the existing private host getter.
#[no_mangle]pub extern "C" fn wpg_hw_game_description()->*const c_void{(&DESCRIPTION as *const Description).cast()}
// Existing host adapter's settings-file getter,from the same source immutable description.
// H8 host policy: bind one stable private path BEFORE wpg_run; unbound source path is unchanged.
#[cfg(feature="host")]
static HOST_SETTINGS:std::sync::OnceLock<std::ffi::CString>=std::sync::OnceLock::new();
#[cfg(feature="host")]
#[no_mangle]pub unsafe extern "C" fn wpg_settings_path_bind(path:*const c_char)->bool{
 if path.is_null(){return false;}
 let path=std::ffi::CStr::from_ptr(path);
 if path.to_bytes().is_empty(){return false;}
 HOST_SETTINGS.set(path.to_owned()).is_ok()
}
#[no_mangle]pub extern "C" fn wpg_hw_settings_file()->*const c_char{
 #[cfg(feature="host")] if let Some(path)=HOST_SETTINGS.get(){return path.as_ptr();}
 DESCRIPTION.settings_file
}
