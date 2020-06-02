//! Future types
//!
use super::controller::Controller;
use futures::ready;
use pin_project::pin_project;
use std::time::Instant;
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::sync::OwnedSemaphorePermit;

/// Future for the `AutoConcurrencyLimit` service.
#[pin_project]
#[derive(Debug)]
pub struct ResponseFuture<F, L> {
    #[pin]
    inner: F,
    // Keep this around so that it is dropped when the future completes
    _permit: OwnedSemaphorePermit,
    controller: Arc<Controller<L>>,
    start: Instant,
}

impl<F, L> ResponseFuture<F, L> {
    pub(super) fn new(
        inner: F,
        _permit: OwnedSemaphorePermit,
        controller: Arc<Controller<L>>,
    ) -> Self {
        Self {
            inner,
            _permit,
            controller,
            start: Instant::now(),
        }
    }
}

impl<F, L, T, E> Future for ResponseFuture<F, L>
where
    F: Future<Output = Result<T, E>>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let future = self.project();
        let output = ready!(future.inner.poll(cx));
        future.controller.adjust_to_response(*future.start);
        Poll::Ready(output)
    }
}
