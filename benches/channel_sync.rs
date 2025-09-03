mod utils;

use std::{hint::black_box, thread::available_parallelism, time::Duration};

use barter_integration::channel::Tx;
use criterion::*;
use utils::{BENCH_MSG_COUNT, check_value, evenly_distribute};

macro_rules! run_bench {
    ($b:expr, $writers:expr, $make_chan:expr, $recv_one:expr, $send_one:expr) => {{
        use std::thread::spawn;

        let readers_dist = evenly_distribute(BENCH_MSG_COUNT, 1);
        let writers_dist = evenly_distribute(BENCH_MSG_COUNT, $writers);

        $b.iter(|| {
            #[allow(unused_mut)]
            let (tx, mut rx) = $make_chan();

            let mut handles = Vec::with_capacity(1 + $writers);

            {
                let iterations = readers_dist[0];
                handles.push(spawn(move || {
                    let mut rx = rx;
                    for _ in 0..iterations {
                        // $recv_one: &mut Rx -> T
                        check_value(black_box($recv_one(&mut rx)));
                    }
                }));
            }

            for d in 0..$writers {
                let tx = tx.clone();
                let iterations = writers_dist[d];
                handles.push(spawn(move || {
                    for i in 0..iterations {
                        // $send_one: &Tx, usize -> ()
                        $send_one(&tx, i + 1);
                    }
                }));
            }

            for h in handles {
                h.join().unwrap();
            }
        })
    }};
}

macro_rules! run_bench_kanal {
    ($b:expr, $writers:expr) => {
        run_bench!(
            $b,
            $writers,
            || kanal::unbounded::<usize>(),
            |rx: &mut kanal::Receiver<usize>| rx.recv().unwrap(),
            |tx: &kanal::Sender<usize>, v: usize| {
                tx.send(v).unwrap();
            }
        )
    };
}

macro_rules! run_bench_crossbeam {
    ($b:expr, $writers:expr) => {
        run_bench!(
            $b,
            $writers,
            || crossbeam_channel::unbounded::<usize>(),
            |rx: &mut crossbeam_channel::Receiver<usize>| rx.recv().unwrap(),
            |tx: &crossbeam_channel::Sender<usize>, v: usize| {
                tx.send(v).unwrap();
            }
        )
    };
}

macro_rules! run_bench_barter {
    ($b:expr, $writers:expr) => {
        run_bench!(
            $b,
            $writers,
            || barter_integration::channel::mpsc_unbounded::<usize>(),
            |rx: &mut barter_integration::channel::UnboundedRx<usize>| rx.next().unwrap(),
            |tx: &barter_integration::channel::UnboundedTx<usize>, v: usize| {
                tx.send(v).unwrap();
            }
        )
    };
}

fn mpsc(c: &mut Criterion) {
    let mut g = c.benchmark_group("sync::mpsc");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));

    g.bench_function("bn_kanal", |b| {
        let core_count = usize::from(available_parallelism().unwrap());
        run_bench_kanal!(b, core_count);
    });
    g.bench_function("bn_crossbeam", |b| {
        let core_count = usize::from(available_parallelism().unwrap());
        run_bench_crossbeam!(b, core_count);
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
        run_bench_kanal!(b, 1);
    });
    g.bench_function("bn_crossbeam", |b| {
        run_bench_crossbeam!(b, 1);
    });
    g.bench_function("bn_barter", |b| {
        run_bench_barter!(b, 1);
    });

    g.finish();
}

criterion_group!(sync_bench, mpsc, spsc);
criterion_main!(sync_bench);
