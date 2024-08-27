use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub trait AsyncRuntime: Send + Sync + 'static {
    type Mutex<T: Send + Sync>: AsyncMutex<T>;
    type RwLock<T: Send + Sync>: AsyncRwLock<T>;
    type JoinAllFuture<I>: Future<Output = Vec<<I::Item as Future>::Output>> + Send
    where
        I: IntoIterator,
        I::Item: Future + Send + 'static,
        <I::Item as Future>::Output: Send,
        I::IntoIter: Send;

        fn spawn<F>(&self, future: F) -> Pin<Box<dyn Future<Output = F::Output> + Send + 'static>>
        where
            F: Future + Send + 'static,
            F::Output: Send + 'static;

    fn sleep(&self, duration: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + Sync>>;

    fn new_mutex<T: Send + Sync + 'static>(&self, t: T) -> Self::Mutex<T>;

    fn new_rwlock<T: Send + Sync + 'static>(&self, t: T) -> Self::RwLock<T>;

    fn join_all<I>(&self, futures: I) -> Self::JoinAllFuture<I>
    where
        I: IntoIterator,
        I::Item: Future + Send + 'static,
        <I::Item as Future>::Output: Send,
        I::IntoIter: Send;
}

pub trait AsyncMutex<T: ?Sized>: Send + Sync {
    type Guard<'a>: AsyncMutexGuard<'a, T> where Self: 'a, T: 'a;
    
    fn lock<'a>(&'a self) -> Pin<Box<dyn Future<Output = Self::Guard<'a>> + Send + Sync + 'a>>;
}

pub trait AsyncMutexGuard<'a, T: ?Sized + 'a>: Send + Sync {
    fn deref(&self) -> &T;
    fn deref_mut(&mut self) -> &mut T;
}

pub trait AsyncRwLock<T: ?Sized>: Send + Sync {
    type ReadGuard<'a>: AsyncRwLockReadGuard<'a, T> where Self: 'a, T: 'a;
    type WriteGuard<'a>: AsyncRwLockWriteGuard<'a, T> where Self: 'a, T: 'a;
    
    fn read<'a>(&'a self) -> Pin<Box<dyn Future<Output = Self::ReadGuard<'a>> + Send + Sync + 'a>>;
    fn write<'a>(&'a self) -> Pin<Box<dyn Future<Output = Self::WriteGuard<'a>> + Send + Sync + 'a>>;
}

pub trait AsyncRwLockReadGuard<'a, T: ?Sized + 'a>: Send + Sync{
    fn deref(&self) -> &T;
}

pub trait AsyncRwLockWriteGuard<'a, T: ?Sized + 'a>: Send + Sync {
    fn deref(&self) -> &T;
    fn deref_mut(&mut self) -> &mut T;
}
