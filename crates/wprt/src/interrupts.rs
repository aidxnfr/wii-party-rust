// SPDX-License-Identifier: GPL-3.0-or-later
// Port of engine/src/core/interrupts.cpp (non-WP_TRACE delivery).
use crate::{cpu::Cpu,memory::*,runtime,dispatch,threads,ffi};
use std::{sync::{OnceLock,atomic::{AtomicI64,Ordering}},time::{Duration,Instant}};
// H5: same opt-in progress clock as the private C++ oracle; ordinary runtime remains unchanged.
fn clock_now()->Instant{
    #[cfg(feature="comparison")] if unsafe{ffi::wphw_comparison_enabled()}{
        static ORIGIN:OnceLock<Instant>=OnceLock::new();let ticks=unsafe{ffi::wphw_comparison_ticks()};
        return *ORIGIN.get_or_init(Instant::now)+Duration::from_nanos(ticks/60750000*1000000000+ticks%60750000*1000000000/60750000);
    }
    Instant::now()
}
static mut IN_INTERRUPT:bool=false;
static IDLE_NANOSECONDS:AtomicI64=AtomicI64::new(0);
static mut NEXT_RETRACE:Option<Instant>=None;
// Port of pristine namespace-scope Clock::now()+kPalPeriod initializer at DLL/executable load.
extern "C" fn initialize_clock(){unsafe{NEXT_RETRACE=Some(clock_now()+Duration::from_micros(20000));}}
#[cfg(windows)] #[used] #[link_section=".CRT$XCU"]
static CLOCK_INITIALIZER:extern "C" fn()=initialize_clock;
// port of engine/src/core/interrupts.cpp retrace_period.
fn retrace_period()->Duration{Duration::from_micros(if rd16(0xcc002002)&0x300==0x100{20000}else{16683})}
// port of engine/src/core/interrupts.cpp arm_video_interrupt.
fn arm_video_interrupt()->bool{let mut armed=false;for address in [0xcc002030,0xcc002034]{let value=rd16(address);if value&0x1000!=0{wr16(address,value|0x8000);armed=true;}}armed}
// port of engine/src/core/interrupts.cpp deliver_decrementer; setjmp uses the accepted token unwind.
fn deliver_decrementer(c:&mut Cpu){
    dispatch::decrementer_fired();static LOG:OnceLock<bool>=OnceLock::new();
    if *LOG.get_or_init(||unsafe{!libc::getenv(c"WP_LOG_IRQ".as_ptr()).is_null()}){eprintln!("IRQ decrementer at tick {}",dispatch::time_base());}
    let handler=rd32(0x80003000+4*8);let context=rd32(0x800000d4);if handler==0||context==0{return;}
    let interrupted=c.clone();c.r[3]=context;let point=threads::save_context(c);threads::enter_exception_context(context);
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
        *c=interrupted.clone();c.spr[26]=interrupted.lr;c.spr[27]=interrupted.msr;c.msr&=!0x8000;c.r[3]=8;c.r[4]=context;
        unsafe{IN_INTERRUPT=true;}runtime::call(c,handler);
    }));
    if let Err(p)=result{runtime::catch_resume(p,point,1);}
    threads::forget_saved_context(context,point);*c=interrupted;unsafe{IN_INTERRUPT=false;}
}
// port of engine/src/core/interrupts.cpp deliver_external_interrupt.
fn deliver_external_interrupt(c:&mut Cpu,index:u32){let handler=rd32(0x80003040+4*index);if handler==0{return;}let saved=c.clone();unsafe{IN_INTERRUPT=true;}c.msr&=!0x8000;c.r[3]=index;c.r[4]=rd32(0x800000d4);runtime::call(c,handler);*c=saved.clone();unsafe{IN_INTERRUPT=false;}c.r[3]=0;runtime::call(c,dispatch::symbol_address("OSSelectThread"));*c=saved;}
// port of engine/src/core/interrupts.cpp deliver_video_interrupt.
fn deliver_video_interrupt(c:&mut Cpu){let handler=rd32(0x80003040+4*24);if handler==0||!arm_video_interrupt(){return;}let saved=c.clone();unsafe{IN_INTERRUPT=true;}c.msr&=!0x8000;c.r[3]=24;c.r[4]=rd32(0x800000d4);runtime::call(c,handler);*c=saved.clone();unsafe{ffi::wphw_video_present();}
#[cfg(feature="comparison")] unsafe{ffi::wphw_comparison_frame(c);}
unsafe{IN_INTERRUPT=false;}c.r[3]=0;runtime::call(c,dispatch::symbol_address("OSSelectThread"));*c=saved;}
// port of engine/src/core/interrupts.cpp idle_share.
pub fn idle_share()->f64{static mut PREVIOUS:Option<Instant>=None;unsafe{if (*std::ptr::addr_of!(PREVIOUS)).is_none(){PREVIOUS=Some(clock_now());}let now=clock_now();let elapsed=now.duration_since(PREVIOUS.unwrap()).as_nanos()as f64;PREVIOUS=Some(now);let idle=IDLE_NANOSECONDS.swap(0,Ordering::SeqCst)as f64;if elapsed>0.0{1.0f64.min(idle/elapsed)}else{-1.0}}}
// port of engine/src/core/interrupts.cpp interrupt_left.
pub fn interrupt_left(){unsafe{IN_INTERRUPT=false;}}
// port of engine/src/core/interrupts.cpp poll_interrupts; native call and delivery order retained.
pub fn poll_interrupts(c:&mut Cpu){unsafe{
    #[cfg(feature="comparison")] ffi::wphw_comparison_advance();
    static INTERVAL:OnceLock<u32>=OnceLock::new();let interval=*INTERVAL.get_or_init(||{let setting=libc::getenv(c"WP_POLL_INTERVAL".as_ptr());let value=if setting.is_null(){0}else{libc::strtoul(setting,std::ptr::null_mut(),10)as u32};if value!=0{value}else{1<<10}});
    *ffi::wphw_poll_counter_ptr()=interval;ffi::wphw_audio_update();ffi::wphw_dsp_update();crate::modules::watch_modules();ffi::wphw_ipc_update();
    if IN_INTERRUPT||c.msr&0x8000==0{ffi::wphw_gx_process();return;}
    if dispatch::decrementer_due(){deliver_decrementer(c);}
    if ffi::wphw_ipc_pending(){deliver_external_interrupt(c,27);}
    for _ in 0..3{let index=ffi::wphw_dsp_pending();if index==0{break;}deliver_external_interrupt(c,index);}
    if ffi::wphw_gx_take_finish(){deliver_external_interrupt(c,19);}
    for _ in 0..3{let index=ffi::wphw_exi_pending();if index==0{break;}deliver_external_interrupt(c,index);}
    if ffi::wphw_si_pending(){deliver_external_interrupt(c,20);}
    ffi::wphw_gx_process();let now=clock_now();static mut PREVIOUS_POLL:Option<Instant>=None;static mut PREVIOUS_IDLE:bool=false;
    if (*std::ptr::addr_of!(PREVIOUS_POLL)).is_none(){PREVIOUS_POLL=Some(now);}let idle=rd32(0x800000e4)==0;
    if idle&&PREVIOUS_IDLE{IDLE_NANOSECONDS.fetch_add(now.duration_since(PREVIOUS_POLL.unwrap()).as_nanos()as i64,Ordering::SeqCst);}
    #[cfg(feature="idle-nap")] let was_idle=PREVIOUS_IDLE;
    PREVIOUS_POLL=Some(now);PREVIOUS_IDLE=idle;
    let next=NEXT_RETRACE.expect("interrupt clock initializer not run");if now<next{
        // Claude OPT A3b: nap only after the guest has been idle continuously for WP_IDLE_NAP_AFTER_US (default 1500 us; short
        // idles while loading/syncing stay busy-polled), and never into the last 500 us before the VI deadline.
        #[cfg(feature="idle-nap")] {static mut IDLE_SINCE:Option<Instant>=None;
            if !(idle&&was_idle){IDLE_SINCE=if idle{Some(now)}else{None};}
            else if let (Some(slice),Some(since))=(idle_nap_slice(),IDLE_SINCE){
                let left=next-now;if now.duration_since(since)>=idle_nap_threshold(now)&&left>Duration::from_micros(700){std::thread::sleep((left-Duration::from_micros(500)).min(slice));}
            }}
        return;}
    let period=retrace_period();let next=next+period;NEXT_RETRACE=Some(if next<now{now+period}else{next});ffi::wphw_si_poll();deliver_video_interrupt(c);
}}

// Claude OPT A3: idle nap slice (normal play only). None = keep the original busy-spin.
#[cfg(feature="idle-nap")]
fn idle_nap_slice()->Option<Duration>{static SLICE:OnceLock<Option<Duration>>=OnceLock::new();*SLICE.get_or_init(||unsafe{
    #[cfg(feature="comparison")] if ffi::wphw_comparison_enabled(){return None;}
    if !libc::getenv(c"WP_IDLE_SPIN".as_ptr()).is_null(){return None;}
    let setting=libc::getenv(c"WP_IDLE_NAP_US".as_ptr());let us=if setting.is_null(){250}else{libc::strtoul(setting,std::ptr::null_mut(),10)as u64};
    if us==0{None}else{Some(Duration::from_micros(us.min(4000)))}
})}
// Claude OPT R6: disc/NAND (non-Bluetooth) IPC activity keeps the long A3b threshold for 100 ms (loading stays
// busy-polled); otherwise nap after WP_IDLE_NAP_QUIET_US (default 300 us) of continuous guest idle.
#[cfg(feature="idle-nap")] static IO_ACTIVITY_NS:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
#[cfg(feature="idle-nap")] fn io_origin()->Instant{static O:OnceLock<Instant>=OnceLock::new();*O.get_or_init(Instant::now)}
#[cfg(feature="idle-nap")] pub fn note_io_activity(){IO_ACTIVITY_NS.store(Instant::now().duration_since(io_origin()).as_nanos()as u64|1,Ordering::Relaxed);}
#[cfg(feature="idle-nap")]
fn idle_nap_threshold(now:Instant)->Duration{
    let last=IO_ACTIVITY_NS.load(Ordering::Relaxed);
    let recent=last!=0&&now.saturating_duration_since(io_origin()).as_nanos()as u64<=last+100_000_000;
    if recent{idle_nap_after()}else{idle_nap_quiet()}
}
#[cfg(feature="idle-nap")]
fn idle_nap_quiet()->Duration{static Q:OnceLock<Duration>=OnceLock::new();*Q.get_or_init(||unsafe{let v=libc::getenv(c"WP_IDLE_NAP_QUIET_US".as_ptr());Duration::from_micros(if v.is_null(){300}else{libc::strtoul(v,std::ptr::null_mut(),10)as u64})})}
#[cfg(feature="idle-nap")]
fn idle_nap_after()->Duration{static AFTER:OnceLock<Duration>=OnceLock::new();*AFTER.get_or_init(||unsafe{let v=libc::getenv(c"WP_IDLE_NAP_AFTER_US".as_ptr());Duration::from_micros(if v.is_null(){1500}else{libc::strtoul(v,std::ptr::null_mut(),10)as u64})})}
