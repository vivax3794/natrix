//! Panic handling

/// Mark that a panic has happened
static PANIC_HAPPENED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

thread_local! {}

/// Has a panic occurred
/// This is only needed for you to call if you are using custom callbacks passed to js.
/// All natrix event handlers already check this.
/// And all uses of `ctx.use_async` uses some magic to insert a check to this *after every*
/// await.
pub fn has_panicked() -> bool {
    let result = PANIC_HAPPENED.load(std::sync::atomic::Ordering::Relaxed);
    if result {
        log::warn!("Access to framework state was attempted after a panic.");
    }
    result
}

/// Set the panic hook to mark that a panic has happened
pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(move |info| {
        let already_paniced = PANIC_HAPPENED.fetch_or(true, std::sync::atomic::Ordering::Relaxed);

        let panic_message = info.to_string();
        log::error!("{panic_message}");

        if already_paniced {
            log::warn!("Panic occured after panic already happend");
            return;
        }

        let msg = if cfg!(debug_assertions) {
            "Panic Occured, check browser console."
        } else {
            "Unknown error occured, please reload the tab."
        };
        if let Err(err) = crate::get_window().alert_with_message(msg) {
            log::error!("Failed to create panic alert {err:?}");
        }
    }));
}

/// Returns if a panic has happened
macro_rules! return_if_panic {
    ($val:expr) => {
        if $crate::panics::has_panicked() {
            return $val;
        }
    };
    () => {
        if $crate::panics::has_panicked() {
            return;
        }
    };
}
pub(crate) use return_if_panic;
