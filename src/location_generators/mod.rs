mod uniform_random;
pub use self::uniform_random::UniformRandomLocationGenerator;
mod array;
pub use self::array::ArrayLocationGenerator;

pub trait LocationGenerator<T> {
    fn next_location(&mut self) -> Option<T>;
}
