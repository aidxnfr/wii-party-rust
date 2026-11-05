// SPDX-License-Identifier: GPL-3.0-or-later
// PRIVATE source-file ports;no game logic/data emitted into the public runtime repository.

use std::arch::x86_64::*;
// Native Itanium ABI names read from nm on the current accepted-source fpu.cpp object.
#[export_name="_ZN2wp14g_hardware_fmaE"]pub static mut HARDWARE_FMA:bool=false;
// port of core/fpu.cpp g_hardware_fma DLL initialization;before any source guest entry.
unsafe extern "C" fn initialize(){HARDWARE_FMA=std::is_x86_feature_detected!("fma");}
#[used]#[link_section=".CRT$XCU"]static INITIALIZE:unsafe extern "C" fn()=initialize;
// port of core/fpu.cpp hardware_fma. GCC's actual vfmadd132sd operand order determines NaN payload selection.
// LLVM lowered the intrinsic to vfmadd213sd:72/8192 source-pure cases differed when both multiplicands were NaNs.
// Preserve the source instruction order,not a NaN normalization. Accepted cpu.rs RV1 is unchanged.
#[target_feature(enable="fma")]
#[export_name="_ZN2wp12hardware_fmaEddd"]pub unsafe extern "C" fn hardware_fma(mut a:f64,b:f64,c:f64)->f64{
 std::arch::asm!("vfmadd132sd xmm0, xmm2, xmm1",inout("xmm0") a,in("xmm1") b,in("xmm2") c,options(nomem,nostack));a
}




