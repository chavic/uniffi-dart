//! Experimental native dispatch for this fixture's exact ABI only.
//!
//! Scope: synchronous calls, one Dart isolate, callbacks bounded by the call.
//! No callback may outlive its relay. No locks are held across Dart execution.
use super::UniFfiTraitVtableSink;
use std::{
    cell::Cell,
    collections::HashMap,
    ptr::NonNull,
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc, Arc, Mutex, OnceLock,
    },
    thread::{self, ThreadId},
};
use uniffi::{Handle, RustBuffer, RustCallStatus};

type Job = Box<dyn FnOnce() + Send>;
enum Event {
    Callback(Job),
    Done,
}
struct Relay {
    owner: ThreadId,
    send: mpsc::Sender<Event>,
}
static ORIGINAL: OnceLock<UniFfiTraitVtableSink> = OnceLock::new();
static ROUTES: OnceLock<Mutex<HashMap<u64, Arc<Relay>>>> = OnceLock::new();
static CALLS: AtomicU64 = AtomicU64::new(0);
static CLONES: AtomicU64 = AtomicU64::new(0);
static FREES: AtomicU64 = AtomicU64::new(0);
static ACTIVE: AtomicU64 = AtomicU64::new(0);
thread_local! { static OWNER_DEPTH: Cell<u32> = const { Cell::new(0) }; }

fn routes() -> &'static Mutex<HashMap<u64, Arc<Relay>>> {
    ROUTES.get_or_init(Default::default)
}
fn route(handle: u64) -> Arc<Relay> {
    routes().lock().unwrap().get(&handle).expect("callback outside active relay").clone()
}
fn dispatch<R: Send + 'static>(relay: &Relay, call: impl FnOnce() -> R + Send + 'static) -> R {
    if thread::current().id() == relay.owner {
        return call();
    }
    let (send, receive) = mpsc::sync_channel(1);
    relay
        .send
        .send(Event::Callback(Box::new(move || {
            send.send(call()).unwrap();
        })))
        .unwrap();
    receive.recv().expect("owner stopped before answering callback")
}

extern "C" fn bytes(handle: u64, value: RustBuffer, out: &mut u64, status: &mut RustCallStatus) {
    CALLS.fetch_add(1, Ordering::Relaxed);
    let relay = route(handle);
    let (result, call_status) = dispatch(&relay, move || {
        let mut result = 0;
        let mut call_status = RustCallStatus::default();
        (ORIGINAL.get().unwrap().bytes)(handle, value, &mut result, &mut call_status);
        (result, call_status)
    });
    *out = result;
    *status = call_status;
}
extern "C" fn checked(handle: u64, value: u32, out: &mut u32, status: &mut RustCallStatus) {
    CALLS.fetch_add(1, Ordering::Relaxed);
    let relay = route(handle);
    let (result, call_status) = dispatch(&relay, move || {
        let mut result = 0;
        let mut call_status = RustCallStatus::default();
        (ORIGINAL.get().unwrap().checked)(handle, value, &mut result, &mut call_status);
        (result, call_status)
    });
    *out = result;
    *status = call_status;
}
extern "C" fn clone_handle(handle: u64) -> u64 {
    CLONES.fetch_add(1, Ordering::Relaxed);
    let relay = route(handle);
    let new_handle = dispatch(&relay, move || (ORIGINAL.get().unwrap().uniffi_clone)(handle));
    assert!(routes().lock().unwrap().insert(new_handle, relay).is_none());
    new_handle
}
extern "C" fn free(handle: u64) {
    FREES.fetch_add(1, Ordering::Relaxed);
    let relay = route(handle);
    dispatch(&relay, move || (ORIGINAL.get().unwrap().uniffi_free)(handle));
    assert!(routes().lock().unwrap().remove(&handle).is_some());
}

// Copy the generated Dart function pointers, retaining the ordinary callback
// implementations. Install native proxies in the actual UniFFI Rust vtable.
#[no_mangle]
pub unsafe extern "C" fn relay_init_callback_vtable_sink(vtable: NonNull<UniFfiTraitVtableSink>) {
    let original = vtable.as_ref();
    assert!(ORIGINAL
        .set(UniFfiTraitVtableSink {
            uniffi_free: original.uniffi_free,
            uniffi_clone: original.uniffi_clone,
            bytes: original.bytes,
            checked: original.checked,
        })
        .is_ok());
    static PROXY: UniFfiTraitVtableSink =
        UniFfiTraitVtableSink { uniffi_free: free, uniffi_clone: clone_handle, bytes, checked };
    super::uniffi_callback_thread_relay_fn_init_callback_vtable_sink(NonNull::from(&PROXY));
}

struct ActiveGuard;
impl Drop for ActiveGuard {
    fn drop(&mut self) {
        OWNER_DEPTH.with(|depth| depth.set(depth.get() - 1));
        ACTIVE.fetch_sub(1, Ordering::Relaxed);
    }
}
struct DoneGuard(mpsc::Sender<Event>);
impl Drop for DoneGuard {
    fn drop(&mut self) {
        let _ = self.0.send(Event::Done);
    }
}
fn run<R: Send + 'static>(
    handle: Handle,
    work: impl FnOnce(Handle) -> (R, RustCallStatus) + Send + 'static,
) -> (R, RustCallStatus) {
    let (send, receive) = mpsc::channel();
    let relay = Arc::new(Relay { owner: thread::current().id(), send: send.clone() });
    assert!(routes().lock().unwrap().insert(handle.as_raw(), relay).is_none());
    ACTIVE.fetch_add(1, Ordering::Relaxed);
    OWNER_DEPTH.with(|depth| depth.set(depth.get() + 1));
    let _active = ActiveGuard;
    let worker = thread::spawn(move || {
        let _done = DoneGuard(send);
        work(handle)
    });
    while let Event::Callback(call) = receive.recv().expect("relay channel closed") {
        call();
    }
    worker.join().expect("fixture operation panicked outside UniFFI error handling")
}

macro_rules! relay_call {
    ($name:ident, $original:ident, $return:ty $(, $arg:ident: $ty:ty)*) => {
        #[no_mangle]
        pub extern "C" fn $name(handle: Handle, $($arg: $ty,)* status: &mut RustCallStatus) -> $return {
            let (result, call_status) = run(handle, move |handle| {
                let mut call_status = RustCallStatus::default();
                let result = super::$original(handle, $($arg,)* &mut call_status);
                (result, call_status)
            });
            *status = call_status;
            result
        }
    };
}
relay_call!(relay_call_bytes_direct, uniffi_callback_thread_relay_fn_func_call_bytes_direct, u64);
relay_call!(relay_call_bytes_thread, uniffi_callback_thread_relay_fn_func_call_bytes_thread, u64);
relay_call!(
    relay_call_bytes_parallel,
    uniffi_callback_thread_relay_fn_func_call_bytes_parallel,
    u64
);
relay_call!(relay_call_clone_thread, uniffi_callback_thread_relay_fn_func_call_clone_thread, u64);
relay_call!(relay_call_checked_thread, uniffi_callback_thread_relay_fn_func_call_checked_thread, u32, value: u32);

#[no_mangle]
pub extern "C" fn relay_stat(which: u32) -> u64 {
    match which {
        0 => routes().lock().unwrap().len() as u64,
        1 => ACTIVE.load(Ordering::Relaxed),
        2 => CALLS.load(Ordering::Relaxed),
        3 => CLONES.load(Ordering::Relaxed),
        4 => FREES.load(Ordering::Relaxed),
        5 => OWNER_DEPTH.with(|depth| depth.get() as u64),
        _ => u64::MAX,
    }
}
