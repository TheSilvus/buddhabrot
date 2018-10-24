use num::complex::Complex64;

pub fn complex_between(a: Complex64, z: Complex64, b: Complex64) -> bool {
    a.re < z.re && z.re < b.re && a.im < z.im && z.im < b.im
}

pub fn calculate_iteration_values(
    function: &Fn(Complex64) -> Complex64,
    initial: Complex64,
    bailout_min: Complex64,
    bailout_max: Complex64,
    max_iterations: u64,
) -> Vec<Complex64> {
    let mut results = vec![];
    let mut z = initial;

    let mut iterations = 0;

    while complex_between(bailout_min, z, bailout_max) && iterations < max_iterations {
        z = function(z);
        results.push(z);
        iterations += 1;
    }

    results
}
