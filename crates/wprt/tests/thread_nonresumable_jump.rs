// H3 acceptance branch: pristine threads.cpp long_jump discards a stale record and creates a fiber.
use wprt::{cpu::Cpu,memory::*,runtime,threads};
fn guest_resume(c:&mut Cpu){
    assert_eq!(runtime::take_resume_address(),0x80005000);assert_eq!(c.r[3],7);
    wr32(0x80003000,c.r[13]);c.r[3]=0x80001000;threads::load_context(c);
    panic!("guest returned after switching to its saved caller");
}
#[test]
fn long_jump_nonresumable(){
    init_sized(0x20000);let mut c=Cpu::default();threads::init_threads(&mut c);
    c.r[3]=0x80001000;c.lr=0x80004000;let root=runtime::save_context(&mut c);
    c.r[3]=0x80002000;c.r[1]=0x80008000;c.r[13]=0x9876;c.lr=0x80006000;
    let _stale=runtime::save_jump(&mut c);wr32(0x80002000,0x80005000);
    runtime::register_resume(0x80005000,guest_resume);
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{c.r[3]=0x80002000;c.r[4]=7;threads::long_jump(&mut c);panic!("nonresumable long_jump returned");}));
    assert_eq!(runtime::catch_resume(result.unwrap_err(),root,1),1);runtime::resume_context(&mut c);
    assert_eq!((rd32(0x80003000),c.r[3],c.lr),(0x9876,1,0x80004000));
}
