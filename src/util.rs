//! General utilities for the library

/// General wrapper for tracing function calls
///
/// Takes a mandatory function name, as well as optionally a format string and one or
/// more arguments. This allows tracing function calls with the called arguments. E.g.
/// ```ignore
/// fn f_name(foo, bar) {
///     trace_call!("f_name","called with {:?}, {:?}", foo, bar);
///     ...
/// }
/// ```
/// or
/// ```ignore
/// fn g() {
///     trace_call!("g");
/// }
/// ```
/// which will simply log `g called`.
#[macro_export]
macro_rules! trace_call {
    ($fn:literal) => {
        let _span = trace_span!($fn);
        let _guard = _span.enter();
        trace!("called");
    };
    ($fn:literal, $estr:literal) => {
        let _span = trace_span!($fn);
        let _guard = _span.enter();
        trace!($estr);
    };
    ($fn:literal, $estr:literal, $($arg:ident),+) => {
        let _span = trace_span!($fn);
        let _guard = _span.enter();
        trace!($estr, $($arg),+);
    };
}
