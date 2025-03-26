use nt_core::{Result, storage::ArticleStorage};
use std::future::Future;

pub mod backends;

pub trait StorageBackend: ArticleStorage {
    fn get_error_message() -> &'static str;
    fn new() -> impl Future<Output = Result<Self>> + Send where Self: Sized;
}

pub use backends::*;
