
use num::complex::Complex64;


mod file_aggregator;
pub use self::file_aggregator::FileAggregator;
mod memory_aggregator;
pub use self::memory_aggregator::MemoryAggregator;

pub trait Aggregator {
    fn aggregate(&mut self, c: Complex64);
}