// Cpu must stay layout-identical to the C++ wp::Cpu (shared by pointer with the C++ engine in the hybrid build).
#[test]
fn cpu_layout_matches_cpp() {
    use std::mem::{align_of, offset_of, size_of};
    use wprt::wp::Cpu;
    assert_eq!(size_of::<Cpu>(), 4832); // g++ sizeof(wp::Cpu)
    assert_eq!(align_of::<Cpu>(), 8);
    assert_eq!(offset_of!(Cpu, f), 128);
    assert_eq!(offset_of!(Cpu, cr), 640);
    assert_eq!(offset_of!(Cpu, fpscr), 660);
    assert_eq!(offset_of!(Cpu, spr), 732);
}
