use std::{
    any::TypeId,
    collections::HashMap,
    marker::PhantomData,
    pin::Pin,
    sync::Mutex,
    task::{Context, Poll},
};

use futures_channel::mpsc::{self, UnboundedReceiver, UnboundedSender};
use futures_util::{Stream, StreamExt};
use once_cell::sync::Lazy;
use slab::Slab;

// Do NOT use HashMap<TypeId, Box<dyn Any + Send>>.
//
// On 32-bit ARM, Zig's LLD generates the vtable for `dyn Any` with a zero
// `type_id` function pointer (ARMv7 LLD + Thumb COMDAT vtable bug). Calling
// `downcast_mut` dispatches through that vtable, crashes at 0x00000000.
//
// Instead, store a type-erased `*mut ()` alongside a concrete drop function
// written to heap memory at construction time (not a link-time vtable), so
// every pointer is a valid non-zero Thumb address on all targets.
struct ErasedSenders {
    ptr: *mut (),
    drop_fn: fn(*mut ()),
}

// SAFETY: the underlying Slab<UnboundedSender<T>> is Send because T: Send.
unsafe impl Send for ErasedSenders {}

impl Drop for ErasedSenders {
    fn drop(&mut self) {
        (self.drop_fn)(self.ptr);
    }
}

impl ErasedSenders {
    fn new<T: Send + 'static>(slab: Slab<UnboundedSender<T>>) -> Self {
        let ptr = Box::into_raw(Box::new(slab)) as *mut ();
        Self {
            ptr,
            drop_fn: |p| unsafe { drop(Box::from_raw(p as *mut Slab<UnboundedSender<T>>)) },
        }
    }

    // SAFETY: caller must only call this when the TypeId key in the map
    // matches T — i.e. this ErasedSenders was created with `new::<T>`.
    unsafe fn as_mut<T: 'static>(&mut self) -> &mut Slab<UnboundedSender<T>> {
        unsafe { &mut *(self.ptr as *mut Slab<UnboundedSender<T>>) }
    }
}

static SUBSCRIBERS: Lazy<Mutex<HashMap<TypeId, ErasedSenders>>> =
    Lazy::new(Default::default);

struct BrokerStream<T: Sync + Send + Clone + 'static>(usize, UnboundedReceiver<T>);

impl<T: Sync + Send + Clone + 'static> Drop for BrokerStream<T> {
    fn drop(&mut self) {
        let mut map = SUBSCRIBERS.lock().unwrap();
        if let Some(erased) = map.get_mut(&TypeId::of::<T>()) {
            unsafe { erased.as_mut::<T>().remove(self.0) };
        }
    }
}

impl<T: Sync + Send + Clone + 'static> Stream for BrokerStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.1.poll_next_unpin(cx)
    }
}

/// A simple broker based on memory
pub struct SimpleBroker<T>(PhantomData<T>);

impl<T: Sync + Send + Clone + 'static> SimpleBroker<T> {
    /// Publish a message that all subscription streams can receive.
    pub fn publish(msg: T) {
        let mut map = SUBSCRIBERS.lock().unwrap();
        if let Some(erased) = map.get_mut(&TypeId::of::<T>()) {
            let slab = unsafe { erased.as_mut::<T>() };
            for (_, sender) in slab.iter_mut() {
                sender.start_send(msg.clone()).ok();
            }
        }
    }

    /// Subscribe to the message of the specified type and returns a `Stream`.
    pub fn subscribe() -> impl Stream<Item = T> {
        let mut map = SUBSCRIBERS.lock().unwrap();
        let erased = map
            .entry(TypeId::of::<T>())
            .or_insert_with(|| ErasedSenders::new::<T>(Slab::default()));
        let slab = unsafe { erased.as_mut::<T>() };
        let (tx, rx) = mpsc::unbounded();
        let id = slab.insert(tx);
        BrokerStream(id, rx)
    }
}
