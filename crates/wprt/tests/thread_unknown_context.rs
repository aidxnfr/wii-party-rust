// H3 acceptance branch: pristine threads.cpp load_context without a saved record.
use wprt::{cpu::Cpu,memory::*,runtime,threads};
fn guest_entry(c:&mut Cpu){
    assert_eq!((c.r[14],c.msr),(0x2468,0x8000));
    wr32(0x80003000,0x1357);c.r[3]=0x80001000;threads::load_context(c);
    panic!("guest returned after switching to its saved caller");
}
#[test]
fn unknown_context_creates_fiber(){
    init_sized(0x20000);let mut c=Cpu::default();threads::init_threads(&mut c);
    c.r[3]=0x80001000;c.lr=0x80004000;let token=runtime::save_context(&mut c);
    let mut other=Cpu::default();other.r[14]=0x2468;other.lr=0x80005000;other.msr=0x8000;
    threads::store(&other,0x80002000);runtime::register(0x80005000,guest_entry);
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{c.r[3]=0x80002000;threads::load_context(&mut c);panic!("unknown-context load returned");}));
    assert_eq!(runtime::catch_resume(result.unwrap_err(),token,1),1);runtime::resume_context(&mut c);
    assert_eq!((rd32(0x80003000),c.r[3],c.lr),(0x1357,1,0x80004000));
}
