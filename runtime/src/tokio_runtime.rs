use tokio::runtime::{Handle, Runtime};
use tokio::sync::{Mutex, RwLock};
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use futures::future::JoinAll;

use crate::{AsyncMutex, AsyncMutexGuard, AsyncRwLock, AsyncRwLockReadGuard, AsyncRwLockWriteGuard, AsyncRuntime};


pub struct TokioRuntime {
    handle: Handle,
    _runtime: Option<Arc<Runtime>>,
}

impl TokioRuntime {
    pub fn new() -> Self {
        match Handle::try_current() {
            Ok(handle) => Self {
                handle,
                _runtime: None,
            },
            Err(_) => {
                let runtime = Arc::new(
                    tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to create Tokio runtime")
                );
                let handle = runtime.handle().clone();
                Self {
                    handle,
                    _runtime: Some(runtime),
                }
            }
        }
    }

    pub fn new_with_handle(handle: Handle) -> Self {
        Self {
            handle,
            _runtime: None,
        }
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}
impl AsyncRuntime for TokioRuntime {
    type Mutex<T: Send + Sync> = TokioMutex<T>;
    type RwLock<T: Send + Sync> = TokioRwLock<T>;
    type JoinAllFuture<I> = JoinAll<I::Item>
    where
        I: IntoIterator,
        I::Item: Future + Send + 'static,
        <I::Item as Future>::Output: Send,
        I::IntoIter: Send;

    fn spawn<F>(&self, future: F) -> Pin<Box<dyn Future<Output = F::Output> + Send + 'static>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let handle = self.handle.spawn(future);
        Box::pin(async move {
            handle.await.expect("Task panicked")
        })
    }

    fn sleep(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>> {
        Box::pin(tokio::time::sleep(duration))
    }

    fn new_mutex<T: Send + Sync + 'static>(&self, t: T) -> Self::Mutex<T> {
        TokioMutex(Arc::new(Mutex::new(t)))
    }

    fn new_rwlock<T: Send + Sync + 'static>(&self, t: T) -> Self::RwLock<T> {
        TokioRwLock(Arc::new(RwLock::new(t)))
    }

    fn join_all<I>(&self, futures: I) -> Self::JoinAllFuture<I>
    where
        I: IntoIterator,
        I::Item: Future + Send + 'static,
        <I::Item as Future>::Output: Send,
        I::IntoIter: Send
    {
        futures::future::join_all(futures)
    }

}

pub struct TokioMutex<T: ?Sized>(Arc<Mutex<T>>);

impl<T: ?Sized + Send + Sync> AsyncMutex<T> for TokioMutex<T> {
    type Guard<'a> = TokioMutexGuard<'a, T> where Self: 'a;

    fn lock<'a>(&'a self) -> Pin<Box<dyn Future<Output = Self::Guard<'a>> + Send + Sync + 'a>> {
        Box::pin(async move {
            TokioMutexGuard(self.0.lock().await)
        })
    }
}

pub struct TokioMutexGuard<'a, T: ?Sized>(tokio::sync::MutexGuard<'a, T>);

impl<'a, T: ?Sized + Send + Sync> AsyncMutexGuard<'a, T> for TokioMutexGuard<'a, T> {
    fn deref(&self) -> &T {
        self.0.deref()
    }

    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}

pub struct TokioRwLock<T: ?Sized + Send + Sync>(Arc<RwLock<T>>);

impl<T: ?Sized + Send + Sync> AsyncRwLock<T> for TokioRwLock<T> {
    type ReadGuard<'a> = TokioRwLockReadGuard<'a, T> where Self: 'a;
    type WriteGuard<'a> = TokioRwLockWriteGuard<'a, T> where Self: 'a;

    fn read<'a>(&'a self) -> Pin<Box<dyn Future<Output = Self::ReadGuard<'a>> + Send + Sync + 'a>> {
        Box::pin(async move {
            TokioRwLockReadGuard(self.0.read().await)
        })
    }

    fn write<'a>(&'a self) -> Pin<Box<dyn Future<Output = Self::WriteGuard<'a>> + Send + Sync + 'a>> {
        Box::pin(async move {
            TokioRwLockWriteGuard(self.0.write().await)
        })
    }
}

pub struct TokioRwLockReadGuard<'a, T: ?Sized + Send>(tokio::sync::RwLockReadGuard<'a, T>);

impl<'a, T: ?Sized + Send + Sync> AsyncRwLockReadGuard<'a, T> for TokioRwLockReadGuard<'a, T> {
    fn deref(&self) -> &T {
        self.0.deref()
    }
}

pub struct TokioRwLockWriteGuard<'a, T: ?Sized>(tokio::sync::RwLockWriteGuard<'a, T>);

impl<'a, T: ?Sized + Send + Sync> AsyncRwLockWriteGuard<'a, T> for TokioRwLockWriteGuard<'a, T> {
    fn deref(&self) -> &T {
        self.0.deref()
    }

    fn deref_mut(&mut self) -> &mut T {
        self.0.deref_mut()
    }
}