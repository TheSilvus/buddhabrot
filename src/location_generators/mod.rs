mod uniform_random;
pub use self::uniform_random::UniformRandomLocationGenerator;

pub trait LocationGenerator<T> {
    fn next_location(&mut self) -> Option<T>;
}
