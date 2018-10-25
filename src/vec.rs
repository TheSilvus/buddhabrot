pub fn filled_with<T: Clone>(value: T, count: usize) -> Vec<T>{
    let mut vec = Vec::with_capacity(count);

    for _ in 0..count {
        vec.push(value.clone());
    }

    vec
}