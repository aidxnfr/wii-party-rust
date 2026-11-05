// H3 acceptance branch: pristine threads.cpp load_context with a saved current fiber.
use wprt::{cpu::Cpu,memory::*,runtime,threads};
#[test]
fn save_load_same_fiber(){
    init_sized(0x20000);let mut c=Cpu::default();threads::init_threads(&mut c);
    c.r[3]=0x80001000;c.r[14]=0x12345678;c.lr=0x80002000;c.msr=0x8000;
    let token=runtime::save_context(&mut c);assert_eq!(c.r[3],0);
    let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
        c.r[14]=0;c.msr=0;c.r[3]=0x80001000;threads::load_context(&mut c);
        panic!("saved-context load returned");
    }));
    assert_eq!(runtime::catch_resume(result.unwrap_err(),token,1),1);
    runtime::resume_context(&mut c);
    assert_eq!((c.r[3],c.r[14],c.lr,c.msr),(1,0x12345678,0x80002000,0));
    assert_eq!(rd16(0x800011a2)&2,0);
}
