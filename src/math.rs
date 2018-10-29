use num::complex::Complex64;

pub fn complex_between(a: Complex64, z: Complex64, b: Complex64) -> bool {
    a.re < z.re && z.re < b.re && a.im < z.im && z.im < b.im
}

pub trait CalculateNext {
    fn next(&mut self, z: Complex64) -> Complex64;
}

// Note: Combining both calculate_* methods heavily decreases performance
pub fn calculate_bailout_iteration<CN: CalculateNext>(
    next: &mut CN,
    initial: Complex64,
    bailout_min: Complex64,
    bailout_max: Complex64,
    max_iterations: usize,
) -> Option<usize> {
    let mut z = initial;
    let mut iterations = 0;

    while complex_between(bailout_min, z, bailout_max) && iterations < max_iterations {
        let new_z = next.next(z);
        if new_z == z {
            iterations = max_iterations;
            break;
        } else {
            z = new_z;
            iterations += 1;
        }
    }
    if complex_between(bailout_min, z, bailout_max) {
        None
    } else {
        Some(iterations)
    }
}

pub fn calculate_iteration_values<CN: CalculateNext>(
    next: &mut CN,
    initial: Complex64,
    bailout_min: Complex64,
    bailout_max: Complex64,
    min_iterations: usize,
    max_iterations: usize,
    results: &mut Vec<Complex64>,
) {
    let mut z = initial;
    let mut iterations = 0;

    while complex_between(bailout_min, z, bailout_max) && iterations < max_iterations {
        let new_z = next.next(z);
        if new_z == z {
            for _ in iterations..max_iterations {
                //TODO apply min_iterations here
                results.push(z);
            }
            break;
        } else {
            z = new_z;
            if iterations >= min_iterations {
                results.push(z);
            }
            iterations += 1;
        }
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

pub fn is_inside_mandelbrot_bulb(c: Complex64) -> bool {
    let x = c.re;
    let y = c.im;

    let p = (x - 1.0 / 4.0) * (x - 1.0 / 4.0) + y * y;

    x < p - 2.0 * p * p + 1.0 / 4.0 && (x + 1.0) * (x + 1.0) + y * y < 1.0 / 16.0
}
