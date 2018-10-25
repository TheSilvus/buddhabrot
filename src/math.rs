use num::complex::Complex64;

pub fn complex_between(a: Complex64, z: Complex64, b: Complex64) -> bool {
    a.re < z.re && z.re < b.re && a.im < z.im && z.im < b.im
}

pub fn calculate_iteration_values(
    function: &Fn(Complex64) -> Complex64,
    initial: Complex64,
    bailout_min: Complex64,
    bailout_max: Complex64,
    max_iterations: usize,
    results: &mut Vec<Complex64>,
) {
    let mut z = initial;
    let mut iterations = 0;

    // Note: Combining both loops heavily decrases performance (~ +50%). 

    while complex_between(bailout_min, z, bailout_max) && iterations < max_iterations {
        z = function(z);
        iterations += 1;
    }
    if complex_between(bailout_min, z, bailout_max) {
        return;
    }

    z = initial;
    iterations = 0;

    while complex_between(bailout_min, z, bailout_max) && iterations < max_iterations {
        z = function(z);
        results.push(z);
        iterations += 1;
    }
}

pub fn complex_to_image(
    c: Complex64,
    min: Complex64,
    max: Complex64,
    width: u64,
    height: u64,
) -> (u64, u64) {
    (
        ((c.re - min.re) / (max.re - min.re) * width as f64) as u64,
        ((c.im - min.im) / (max.im - min.im) * height as f64) as u64,
    )
}
