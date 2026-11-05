// H3 acceptance branch: pristine threads.cpp long_jump with matching saved link/stack.
use wprt::{cpu::Cpu,memory::*,runtime,threads};
#[test]
fn long_jump_resumable(){
    init_sized(0x20000);let mut c=Cpu::default();threads::init_threads(&mut c);
    c.r[3]=0x80001000;c.r[1]=0x80008000;c.r[13]=0xabc;c.lr=0x80002000;
    let token=runtime::save_jump(&mut c);
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
        c.r[13]=0;c.r[3]=0x80001000;c.r[4]=0;threads::long_jump(&mut c);panic!("resumable long_jump returned");
    }));
    assert_eq!(runtime::catch_resume(result.unwrap_err(),token,1),1);runtime::resume_jump(&mut c);
    assert_eq!((c.r[3],c.r[13],c.r[1],c.lr),(1,0xabc,0x80008000,0x80002000));
}
