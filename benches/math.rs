#[macro_use]
extern crate criterion;
extern crate num;
extern crate rand;

use criterion::Criterion;
use num::complex::Complex64;

const ITERATIONS: usize = 10000;

fn criterion_benchmark(c: &mut Criterion) {
    let random_setup = || {
        let x = rand::random::<f64>() * 4.0 - 2.0;
        let y = rand::random::<f64>() * 4.0 - 2.0;
        let mut buffer = Vec::with_capacity(ITERATIONS * 2);

        for _ in 0..ITERATIONS - 50 {
            buffer.push(Complex64::new(0.0, 0.0));
        }

        (buffer, Complex64::new(x, y))
    };

    c.bench_function("double loop", move |b| {
        b.iter_with_setup(random_setup, |(mut buffer, c)| {
            let mut z = Complex64::new(0.0, 0.0);
            let mut iterations = 0;

            while -2.0 < z.re && z.re < 2.0 && -2.0 < z.im && z.im < 2.0 && iterations < ITERATIONS {
                z = z * z + c;
                iterations += 1;
            }

            if iterations == ITERATIONS {
                return (z, buffer);
            }

            z = Complex64::new(0.0, 0.0);
            iterations = 0;

            while -2.0 < z.re && z.re < 2.0 && -2.0 < z.im && z.im < 2.0 && iterations < ITERATIONS {
                z = z * z + c;
                buffer.push(z);
                iterations += 1;
            }

            (z, buffer)
        })
    });
    c.bench_function("single loop", move |b| {
        b.iter_with_setup(random_setup, |(mut buffer, c)| {
            let mut z = Complex64::new(0.0, 0.0);
            let mut iterations = 0;

            while -2.0 < z.re && z.re < 2.0 && -2.0 < z.im && z.im < 2.0 && iterations < ITERATIONS {
                z = z * z + c;
                buffer.push(z);
                iterations += 1;
            }

            if iterations == ITERATIONS {
                let new_len = buffer.len() - iterations;
                buffer.truncate(new_len);
            }

            (z, buffer)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
