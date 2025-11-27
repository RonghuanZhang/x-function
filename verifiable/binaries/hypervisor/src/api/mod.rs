use axum::Router;

pub mod encrypt;
pub mod execute;
pub mod ping;
pub mod policy;

pub trait ServerState: Clone + Sync + Send + 'static {}
impl<T: Clone + Sync + Send + 'static> ServerState for T {}

pub trait RouterRegister<V> {
    fn register_api<T>(self, _: impl Fn(Router<V>) -> Router<T>) -> Router<T>;
    fn register_x402_api<T>(self, _: V, _: impl Fn(Router<V>, V) -> Router<T>) -> Router<T>;
}

impl<V> RouterRegister<V> for Router<V> {
    fn register_api<T>(self, f: impl Fn(Router<V>) -> Router<T>) -> Router<T> {
        f(self)
    }

    fn register_x402_api<T>(self, state: V, f: impl Fn(Router<V>, V) -> Router<T>) -> Router<T> {
        f(self, state)
    }
}
