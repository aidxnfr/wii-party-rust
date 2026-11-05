pub const SDSP_SIZE:usize=66472; pub const REGS_SIZE:usize=72; pub const ATOMIC_BOOL_SIZE:usize=1;
pub const R:usize=0; pub const PC:usize=72; pub const CONTROL_REG:usize=74; pub const INIT_CLEAR_TIME:usize=80; pub const REG_STACK_PTRS:usize=88; pub const EXCEPTIONS:usize=92; pub const EXT_INT_WAITING:usize=93; pub const REG_STACKS:usize=96;
pub const IROM:usize=368; pub const IRAM:usize=352; pub const DRAM:usize=360; pub const COEF:usize=376; pub const STEP_COUNTER:usize=392; pub const ACCELERATOR:usize=912; pub const IFX_REGS:usize=400; pub const MAILBOX:usize=920; pub const ANALYZER:usize=936; pub const DSP_CORE:usize=928;
pub const R_AR:usize=0; pub const R_IX:usize=8; pub const R_WR:usize=16; pub const R_ST:usize=24; pub const R_CR:usize=32; pub const R_SR:usize=34; pub const R_PROD:usize=40; pub const R_AX:usize=48; pub const R_AC:usize=56; pub const AX_SIZE:usize=4; pub const AC_SIZE:usize=8;
pub const CORE_DSP:usize=0; pub const CORE_STATE:usize=132008; pub const CORE_INTERP:usize=132016; pub const CORE_CAP:usize=132024; pub const STATE_STOPPED:u32=0; pub const STATE_RUNNING:u32=1; pub const STATE_STEPPING:u32=2; pub const STATE_SIZE:usize=4;
pub const OPT_IROM:usize=0; pub const OPT_COEF:usize=8192; pub const OPT_CAPTURE:usize=12296;
pub const VT_LOG_IFX_READ:usize=2; pub const VT_LOG_IFX_WRITE:usize=3; pub const VT_LOG_DMA:usize=4;
