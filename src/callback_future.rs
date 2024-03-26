use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

#[derive(Default, Clone)]
pub struct CallbackFuture<T>(Arc<CallbackFutureInner<T>>);

struct CallbackFutureInner<T> {
    waker: RefCell<Option<Waker>>,
    result: RefCell<Option<T>>,
}

impl<T> Default for CallbackFutureInner<T> {
    fn default() -> Self {
        Self {
            waker: RefCell::new(None),
            result: RefCell::new(None),
        }
    }
}

impl<T> CallbackFuture<T> {
    // call this from your callback
    pub fn publish(&self, result: T) {
        self.0.result.replace(Some(result));
        self.0.waker.take().map(|w| w.wake());
    }
}

impl<T> CallbackFuture<T> {
    pub fn new() -> Self {
        Self(Arc::new(CallbackFutureInner::<T>::default()))
    }
}

impl<T> Future for CallbackFuture<T> {
    type Output = T;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.result.take() {
            Some(x) => Poll::Ready(x),
            None => {
                self.0.waker.replace(Some(cx.waker().clone()));
                Poll::Pending
            }
        }
    }
}
