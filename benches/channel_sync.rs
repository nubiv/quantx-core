mod utils;

use std::{hint::black_box, thread::available_parallelism, time::Duration};

use barter_integration::channel::Tx;
use criterion::*;
use utils::{BENCH_MSG_COUNT, check_value, evenly_distribute};

macro_rules! run_bench_kanal {
    ($b:expr, $writers:expr, $readers:expr) => {
        use std::thread::spawn;

        let readers_dist = evenly_distribute(BENCH_MSG_COUNT, $readers);
        let writers_dist = evenly_distribute(BENCH_MSG_COUNT, $writers);

        $b.iter(|| {
            let (tx, rx) = kanal::unbounded::<usize>();
            let mut handles = Vec::with_capacity($readers + $writers);

            for d in 0..$readers {
                let rx = rx.clone();
                let iterations = readers_dist[d];
                handles.push(spawn(move || {
                    for _ in 0..iterations {
                        check_value(black_box(rx.recv().unwrap()));
                    }
                }));
            }

            for d in 0..$writers {
                let tx = tx.clone();
                let iterations = writers_dist[d];
                handles.push(spawn(move || {
                    for i in 0..iterations {
                        tx.send(i + 1).unwrap();
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        })
    };
}

macro_rules! run_bench_barter {
    ($b:expr, $writers:expr) => {
        use std::thread::spawn;

        let readers_dist = evenly_distribute(BENCH_MSG_COUNT, 1);
        let writers_dist = evenly_distribute(BENCH_MSG_COUNT, $writers);

        $b.iter(|| {
            let (tx, mut rx) = barter_integration::channel::mpsc_unbounded::<usize>();
            let mut handles = Vec::with_capacity(1 + $writers);

            {
                let iterations = readers_dist[0];
                handles.push(spawn(move || {
                    for _ in 0..iterations {
                        check_value(black_box(rx.next().unwrap()));
                    }
                }));
            }

            for d in 0..$writers {
                let tx = tx.clone();
                let iterations = writers_dist[d];
                handles.push(spawn(move || {
                    for i in 0..iterations {
                        tx.send(i + 1).unwrap();
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        })
    };
}

fn mpsc(c: &mut Criterion) {
    let mut g = c.benchmark_group("sync::mpsc");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));

    g.bench_function("bn_kanal", |b| {
        let core_count = usize::from(available_parallelism().unwrap());
        run_bench_kanal!(b, core_count, 1);
    });
    g.bench_function("bn_barter", |b| {
        let core_count = usize::from(available_parallelism().unwrap());
        run_bench_barter!(b, core_count);
    });

    g.finish();
}

fn spsc(c: &mut Criterion) {
    let mut g = c.benchmark_group("sync::spsc");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));

    g.bench_function("bn_kanal", |b| {
        run_bench_kanal!(b, 1, 1);
    });
    g.bench_function("bn_barter", |b| {
        run_bench_barter!(b, 1);
    });

    g.finish();
}

criterion_group!(sync_bench, mpsc, spsc);
criterion_main!(sync_bench);
