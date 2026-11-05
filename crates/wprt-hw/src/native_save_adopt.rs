// SPDX-License-Identifier: GPL-3.0-or-later
// AI-assisted port of engine/src/platform/save_backup.cpp adopt_root_files.
// Filesystem/path/string ABI remains native. Native exceptions are captured by
// the leaf and rethrown after Rust returns,never unwound across a Rust C frame.
use std::ffi::c_void;
type Operation=unsafe extern "C" fn(*mut c_void,i32)->i32;
// Operation IDs belong solely to this ABI,not to game logic.
const END:i32=0;const REGULAR:i32=1;const TARGET:i32=2;const EXISTS:i32=3;
const CREATE:i32=4;const RENAME:i32=5;const CLEAR_ERROR:i32=6;const APPEND:i32=7;const INCREMENT:i32=8;
// port of adopt_root_files:preserve regular/exists short circuits,create then
// rename even if create returned an error,append only after successful rename.
#[no_mangle]pub unsafe extern "C" fn wprt_saves_adopt(c:*mut c_void,op:Operation){
    loop{
        let end=op(c,END);if end<0||end!=0{return}
        let regular=op(c,REGULAR);if regular<0{return}
        if regular!=0{
            if op(c,TARGET)<0{return}
            let exists=op(c,EXISTS);if exists<0{return}
            if exists==0{
                if op(c,CREATE)<0{return}
                if op(c,RENAME)<0{return}
                let clear=op(c,CLEAR_ERROR);if clear<0{return}
                if clear!=0&&op(c,APPEND)<0{return}
            }
        }
        if op(c,INCREMENT)<0{return}
    }
}
