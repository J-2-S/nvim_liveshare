use once_cell::sync::Lazy;
use threadpool::ThreadPool;
static THREADS: Lazy<ThreadPool> = Lazy::new(|| ThreadPool::new(3));
fn main() {
    println!("Hello, world!");
}
