// The setjmp replacement: a deeper call unwinds back to the frame that saved, which continues after the site.
use wprt::wp;
fn deep(token: u64, depth: u32) { if depth == 0 { wp::resume_to(token) } deep(token, depth - 1) }
#[test]
fn resume_reaches_owner_frame_and_skips_others() {
    std::panic::set_hook(Box::new(|_| {}));
    let mut log = vec![];
    let outer = wp::new_token();
    let r = std::panic::catch_unwind(|| {
        let inner = wp::new_token();
        let r2 = std::panic::catch_unwind(|| deep(outer, 50));
        match r2 { Ok(()) => 0, Err(p) => wp::catch_resume(p, inner, 1) } // must NOT catch: other token
    });
    match r { Ok(_) => log.push("inner caught"), Err(p) => log.push(if wp::catch_resume(p, outer, 7) == 7 { "outer" } else { "?" }) }
    assert_eq!(log, ["outer"]);
}
