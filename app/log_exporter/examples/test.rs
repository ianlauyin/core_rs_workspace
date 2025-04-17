use std::sync::Arc;

use tokio::sync::Semaphore;

pub fn main() {
    let _semaphore = Arc::new(Semaphore::new(3));
    // semaphore.a
}
