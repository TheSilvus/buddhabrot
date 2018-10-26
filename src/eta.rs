use std::sync::{atomic::AtomicUsize, atomic::Ordering, Arc};
use std::thread;
use std::time::{Duration, Instant};

pub struct ETA {
    eta_store: Arc<ETAStore>,
    current: usize,
    section_total: usize,
}
impl ETA {
    pub fn new(total: usize, section_total: usize, timeout: u64) -> ETA {
        let eta_store = Arc::new(ETAStore {
            start: Instant::now(),
            timeout,
            total,
            current: AtomicUsize::new(0),
        });
        ETAStore::run_thread(eta_store.clone());

        ETA {
            eta_store,
            current: 0,
            section_total,
        }
    }

    pub fn count(&mut self) {
        self.count_n(1);
    }
    pub fn count_n(&mut self, n: usize) {
        self.current += n;

        if self.current >= self.section_total {
            self.eta_store.count(self.current);
            self.current = 0;
        }
    }
}
impl Clone for ETA {
    fn clone(&self) -> ETA {
        ETA {
            eta_store: self.eta_store.clone(),
            current: 0,
            section_total: self.section_total,
        }
    }
}

struct ETAStore {
    start: Instant,
    timeout: u64,
    total: usize,
    current: AtomicUsize,
}
impl ETAStore {
    fn count(&self, n: usize) {
        self.current.fetch_add(n, Ordering::Relaxed);
    }

    fn print(&self) {
        let duration = self.start.elapsed();
        let current = self.current.load(Ordering::Relaxed);

        let duration = duration.as_secs() as f64
            + duration.subsec_millis() as f64 * 1e-3
            + duration.subsec_micros() as f64 * 1e-6;

        let estimated_left = (duration * (self.total as f64 / current as f64) - duration) as u64;
        println!(
            "ETA: {}h{:02}m{:02}s; {} / {}; {:.5}%; {:.2} samples/s",
            estimated_left / (60 * 60),
            (estimated_left / 60) % 60,
            estimated_left % 60,
            current,
            self.total,
            (current as f64 / self.total as f64) * 100.0,
            current as f64 / duration
        );
    }

    fn run_thread(store: Arc<ETAStore>) {
        thread::Builder::new()
            .name("ETA".to_string())
            .spawn(move || while Arc::strong_count(&store) > 1 {
                thread::sleep(Duration::from_millis(store.timeout));

                store.print();
            }).expect("Unable to spawn thread");
    }
}
