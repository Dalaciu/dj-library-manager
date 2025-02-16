use std::sync::atomic::AtomicUsize;
use rayon::prelude::*;

pub trait ParallelProcessor {
    fn init_parallel_processing() {
        // Configure thread pool if not already configured
        if rayon::current_num_threads() == 1 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(num_cpus::get())
                .build_global()
                .expect("Failed to initialize thread pool");
        }
        println!("Using {} CPU threads for processing", rayon::current_num_threads());
    }

    fn get_progress_counter() -> AtomicUsize {
        AtomicUsize::new(0)
    }

    fn process_chunks<T, F, R>(items: Vec<T>, chunk_size: usize, f: F) -> Vec<R>
    where
        T: Send + Sync,
        R: Send,
        F: Fn(&[T]) -> Vec<R> + Send + Sync,
    {
        let chunks: Vec<_> = items.chunks(chunk_size).collect();
        
        chunks.par_iter()
            .enumerate()
            .flat_map(|(chunk_index, chunk)| {
                println!("Processing chunk {}/{} in parallel", chunk_index + 1, chunks.len());
                f(chunk)
            })
            .collect()
    }

    fn parallel_compare<T, F, R>(items: &[T], comparison_fn: F) -> Vec<R>
    where
        T: Sync,
        R: Send,
        F: Fn(&T, &T) -> Option<R> + Send + Sync,
    {
        (0..items.len())
            .into_par_iter()
            .flat_map(|i| {
                let items_slice = &items[i + 1..];
                items_slice
                    .iter()
                    .filter_map(|item2| comparison_fn(&items[i], item2))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}