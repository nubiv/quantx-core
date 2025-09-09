mod utils;

use std::{hint::black_box, thread::available_parallelism, time::Duration};

use quantx_core::channel::{SyncTx, SyncRx};
use barter_integration::channel::Tx;
use criterion::*;
use utils::{BENCH_MSG_COUNT, evenly_distribute};

macro_rules! bench_all_mpsc {
    ($g:ident, $writers:expr, $t:ty, $gen:expr, $check:expr) => {{
        $g.bench_function("bn_kanal", |b| {
            run_bench_kanal!(b, $writers, $t, $gen, $check);
        });
        $g.bench_function("bn_crossbeam", |b| {
            run_bench_crossbeam!(b, $writers, $t, $gen, $check);
        });
        $g.bench_function("bn_barter", |b| {
            run_bench_barter!(b, $writers, $t, $gen, $check);
        });
        $g.bench_function("bn_custom", |b| {
            run_bench_custom!(b, $writers, $t, $gen, $check);
        });
    }};
}


macro_rules! run_bench_kanal {
    ($b:expr, $writers:expr, $t:ty, $gen_val:expr, $check_val:expr) => {
        run_bench!(
            $b,
            $writers,
            || kanal::unbounded::<$t>(),
            |rx: &mut kanal::Receiver<$t>| rx.recv().unwrap(),
            |tx: &kanal::Sender<$t>, v: $t| { tx.send(v).unwrap(); },
            $gen_val,
            $check_val
        )
    };
}

macro_rules! run_bench_crossbeam {
    ($b:expr, $writers:expr, $t:ty, $gen_val:expr, $check_val:expr) => {
        run_bench!(
            $b,
            $writers,
            || crossbeam_channel::unbounded::<$t>(),
            |rx: &mut crossbeam_channel::Receiver<$t>| rx.recv().unwrap(),
            |tx: &crossbeam_channel::Sender<$t>, v: $t| { tx.send(v).unwrap(); },
            $gen_val,
            $check_val
        )
    };
}

macro_rules! run_bench_barter {
    ($b:expr, $writers:expr, $t:ty, $gen_val:expr, $check_val:expr) => {
        run_bench!(
            $b,
            $writers,
            || barter_integration::channel::mpsc_unbounded::<$t>(),
            |rx: &mut barter_integration::channel::UnboundedRx<$t>| rx.next().unwrap(),
            |tx: &barter_integration::channel::UnboundedTx<$t>, v: $t| { tx.send(v).unwrap(); },
            $gen_val,
            $check_val
        )
    };
}

macro_rules! run_bench_custom {
    ($b:expr, $writers:expr, $t:ty, $gen_val:expr, $check_val:expr) => {
        run_bench!(
            $b,
            $writers,
            || quantx_core::channel::mpsc_unbounded::<quantx_core::channel::SyncChannel, $t>(),
            |rx: &mut quantx_core::channel::UnboundedRx<quantx_core::channel::SyncChannel, $t>| rx.recv().unwrap(),
            |tx: &quantx_core::channel::UnboundedTx<quantx_core::channel::SyncChannel, $t>, v: $t| { tx.send(v).unwrap(); },
            $gen_val,
            $check_val
        )
    };
}

macro_rules! run_bench {
    (
        $b:expr,
        $writers:expr,
        $make_chan:expr,           
        $recv_one:expr,            
        $send_one:expr,            
        $gen_val:expr,             
        $check_val:expr            
    ) => {{
        use std::thread::spawn;

        let readers_dist = evenly_distribute(BENCH_MSG_COUNT, 1);
        let writers_dist = evenly_distribute(BENCH_MSG_COUNT, $writers);

        $b.iter(|| {
            #[allow(unused_mut)]    
            let (tx, mut rx) = $make_chan();

            let mut handles = Vec::with_capacity(1 + $writers);

            {
                let iters = readers_dist[0];
                handles.push(spawn(move || {
                    let mut rx = rx;
                    for _ in 0..iters {
                        let v = $recv_one(&mut rx);
                        $check_val(v);
                    }
                }));
            }

            for d in 0..$writers {
                let tx = tx.clone();
                let iters = writers_dist[d];
                handles.push(spawn(move || {
                    for i in 0..iters {
                        let v = $gen_val(i + 1);
                        $send_one(&tx, v);
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
        })
    }};
}

fn mpsc_scalar(c: &mut Criterion) {
    let mut g = c.benchmark_group("sync::mpsc::scalar");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    let core_count = usize::from(available_parallelism().unwrap());
    bench_all_mpsc!(
        g,
        core_count,
        usize,
        |i: usize| i + 1,
        |m: usize| {
            assert!(m != 0);
            black_box(m);
        }
    );

    g.finish();
}

fn spsc_scalar(c: &mut Criterion) {
    let mut g = c.benchmark_group("sync::spsc::scalar");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    bench_all_mpsc!(
        g,
        1,
        usize,
        |i: usize| i + 1,
        |m: usize| {
            assert!(m != 0);
            black_box(m);
        }
    );

    g.finish();
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DummyPod {
    a: u64,
    b: u64,
    c: u64,
    d: u64,
}

fn mpsc_pod(c: &mut Criterion) {
    let mut g = c.benchmark_group("sync::mpsc::pod");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    let core_count = usize::from(available_parallelism().unwrap());
    bench_all_mpsc!(
        g,
        core_count,
        DummyPod,
        |i: usize| DummyPod {
            a: i as u64 + 1, 
            b: !i as u64, 
            c: 0xDEADBEEF, 
            d: 42
        },
        |m: DummyPod| {
            assert!(m.a != 0);
            black_box(m);
        }
    );

    g.finish();
}

fn spsc_pod(c: &mut Criterion) {
    let mut g = c.benchmark_group("sync::spsc::pod");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    bench_all_mpsc!(
        g,
        1,
        DummyPod,
        |i: usize| DummyPod {
            a: i as u64 + 1, 
            b: !i as u64, 
            c: 0xDEADBEEF, 
            d: 42
        },
        |m: DummyPod| {
            assert!(m.a != 0);
            black_box(m);
        }
    );

    g.finish();
}

#[allow(dead_code)]
const ARRAY_SIZE: usize = 128;
#[allow(dead_code)]
static ZERO_128: [u8; ARRAY_SIZE] = [0; ARRAY_SIZE];
fn mpsc_arc(c: &mut Criterion) {
    use std::sync::Arc;
    let mut g = c.benchmark_group("sync::mpsc::arc");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    let core_count = usize::from(available_parallelism().unwrap());
    bench_all_mpsc!(
        g,
        core_count,
        Arc<[u8; ARRAY_SIZE]>,
        |_i: usize| Arc::new(ZERO_128),
        |m: Arc<[u8; ARRAY_SIZE]>| {
            assert!(m[0] == 0);
            black_box(m);
        }
    );

    g.finish();
}

fn spsc_arc(c: &mut Criterion) {
    use std::sync::Arc;
    let mut g = c.benchmark_group("sync::spsc::arc");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    bench_all_mpsc!(
        g,
        1,
        Arc<[u8; ARRAY_SIZE]>,
        |_i: usize| Arc::new(ZERO_128),
        |m: Arc<[u8; ARRAY_SIZE]>| {
            assert!(m[0] == 0);
            black_box(m);
        }
    );

    g.finish();
}

#[allow(dead_code)]
const BUFFER_SIZE: usize = 1024;
fn mpsc_box(c: &mut Criterion) {
    let mut g = c.benchmark_group("sync::mpsc::box");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    let core_count = usize::from(available_parallelism().unwrap());
    bench_all_mpsc!(
        g,
        core_count,
        Box<[u8; BUFFER_SIZE]>,
        |_i: usize| Box::new([0u8; BUFFER_SIZE]),
        |m: Box<[u8; BUFFER_SIZE]>| {
            assert!(m[0] == 0);
            black_box(m);
        }
    );

    g.finish();
}

fn spsc_box(c: &mut Criterion) {
    let mut g = c.benchmark_group("sync::spsc::box");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    bench_all_mpsc!(
        g,
        1,
        Box<[u8; BUFFER_SIZE]>,
        |_i: usize| Box::new([0u8; BUFFER_SIZE]),
        |m: Box<[u8; BUFFER_SIZE]>| {
            assert!(m[0] == 0);
            black_box(m);
        }
    );

    g.finish();
}

fn mpsc_event(c: &mut Criterion) {
    use smol_str::SmolStr;
    use barter_data::event::{MarketEvent, DataKind};
    let mut g = c.benchmark_group("sync::mpsc::event");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    let core_count = usize::from(available_parallelism().unwrap());
    let utc_now = chrono::Utc::now();
    bench_all_mpsc!(
        g,
        core_count,
        MarketEvent<SmolStr, DataKind>,
        |_i: usize| MarketEvent { 
            time_exchange: utc_now, 
            time_received: utc_now, 
            exchange: barter_instrument::exchange::ExchangeId::BybitSpot, 
            instrument: SmolStr::new("INSTRUMENT"), 
            kind: DataKind::Trade(barter_data::subscription::trade::PublicTrade {
                id: String::from("12345678"), 
                price: 12345.67,
                amount: 0.00123,
                side: barter_instrument::Side::Buy,
            }), 
        },
        |m: MarketEvent<SmolStr, DataKind>| {
            assert!(m.instrument.len() > 0);
            black_box(m);
        }
    );

    g.finish();
}

fn spsc_event(c: &mut Criterion) {
    use smol_str::SmolStr;
    use barter_data::event::{MarketEvent, DataKind};
    let mut g = c.benchmark_group("sync::spsc::event");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    let utc_now = chrono::Utc::now();
    bench_all_mpsc!(
        g,
        1,
        MarketEvent<SmolStr, DataKind>,
        |_i: usize| MarketEvent { 
            time_exchange: utc_now, 
            time_received: utc_now, 
            exchange: barter_instrument::exchange::ExchangeId::BybitSpot, 
            instrument: SmolStr::new("INSTRUMENT"), 
            kind: DataKind::Trade(barter_data::subscription::trade::PublicTrade {
                id: String::from("12345678"), 
                price: 12345.67,
                amount: 0.00123,
                side: barter_instrument::Side::Buy,
            }), 
        },
        |m: MarketEvent<SmolStr, DataKind>| {
            assert!(m.instrument.len() > 0);
            black_box(m);
        }
    );

    g.finish();
}

criterion_group!(
    sync_bench, 
    mpsc_scalar, 
    spsc_scalar, 
    mpsc_pod, 
    spsc_pod, 
    mpsc_arc, 
    spsc_arc, 
    mpsc_box, 
    spsc_box, 
    mpsc_event, 
    spsc_event
);
criterion_main!(sync_bench);
