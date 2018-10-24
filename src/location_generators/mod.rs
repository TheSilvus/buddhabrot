mod uniform_random;
pub use self::uniform_random::UniformRandomLocationGenerator;

pub trait LocationGenerator<T> {
    fn next_task(&mut self) -> Option<T>;
}
