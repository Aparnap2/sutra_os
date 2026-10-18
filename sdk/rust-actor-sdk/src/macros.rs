//! Helper macros for SūtraOS actor development.

/// Declare the actor's `on_interrupt` entry point.
///
/// # Example
/// ```ignore
/// sutra_actor_main!(my_on_interrupt);
///
/// fn my_on_interrupt(event_json: &str) -> Result<String, String> {
///     // ... build and return NyayaProof JSON
/// }
/// ```
#[macro_export]
macro_rules! sutra_actor_main {
    ($handler:ident) => {
        #[no_mangle]
        pub extern "C" fn on_interrupt(ptr: *const u8, len: usize) -> *mut u8 {
            let json = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len)) };
            let result = $handler(json);
            let out = match result {
                Ok(s) => s,
                Err(e) => format!("{{\"error\":\"{}\"}}", e),
            };
            let boxed = out.into_bytes().into_boxed_slice();
            Box::into_raw(boxed) as *mut u8
        }
    };
}
