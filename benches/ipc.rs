use std::{hint::black_box, time::Duration};

use criterion::*;
use iceoryx2::prelude::*;
use iceoryx2_bb_container::{byte_string::FixedSizeByteString, queue::FixedSizeQueue, vec::FixedSizeVec};
use quantx_core::transport::ipc::ipc_shm;
use utils::BENCH_MSG_COUNT;

mod utils;

fn shm_scalar(c: &mut Criterion) {
    let mut g = c.benchmark_group("shm::scalar");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    let service_name = "shm_scalar";
    let interesting_key = 0;
    let (writer, reader, notifier, listener) = ipc_shm::<u32, u32>(service_name);
    let writer_entry_handle_mut = writer.entry::<u32>(&interesting_key).unwrap();
    let reader_entry_handle = reader.entry::<u32>(&interesting_key).unwrap();
    let writer_entry_id = writer_entry_handle_mut.entry_id();
    let reader_entry_id = reader_entry_handle.entry_id();
    assert_eq!(writer_entry_id, reader_entry_id);
    let entry_id = writer_entry_id;

    g.bench_function("ipc_write_read_scalar", |b| {
        b.iter(|| {
            let mut received_count = 0;

            for i in 0..BENCH_MSG_COUNT {
                let msg: u32 = (i as u32).wrapping_add(1);

                writer_entry_handle_mut.update_with_copy(msg);
                notifier.notify_with_custom_event_id(entry_id).unwrap();

                if let Some(event_id) = listener.blocking_wait_one().unwrap() {
                    assert!(event_id == entry_id);
                    let received = reader_entry_handle.get();
                    assert_eq!(received, msg);
                    black_box(received);
                    received_count += 1;
                    assert_eq!(received_count, i + 1);
                }
            }
        });
    });

    g.finish();
}

#[derive(Debug, Default, PlacementDefault, ZeroCopySend)]
#[repr(C)]
pub struct ComplexData {
    name: FixedSizeByteString<4>,
    data: FixedSizeVec<u64, 4>,
}
#[derive(Debug, Default, PlacementDefault, ZeroCopySend)]
#[repr(C)]
pub struct ComplexDataType {
    plain_old_data: u64,
    text: FixedSizeByteString<8>,
    vec_of_data: FixedSizeVec<u64, 4>,
    vec_of_complex_data: FixedSizeVec<ComplexData, 404857>,
    a_queue_of_things: FixedSizeQueue<FixedSizeByteString<4>, 2>,
}
fn shm_complex(c: &mut Criterion) {
    let mut g = c.benchmark_group("shm::complex");

    g.throughput(Throughput::Elements(BENCH_MSG_COUNT as u64));
    g.sample_size(10).warm_up_time(Duration::from_secs(1));
    g.measurement_time(Duration::from_secs(10));

    let service_name = "shm_complex";
    let interesting_key = 0;
    let (writer, reader, notifier, listener) = ipc_shm::<u32, FixedSizeByteString<8>>(service_name);
    let writer_entry_handle_mut = writer.entry(&interesting_key).unwrap();
    let reader_entry_handle = reader.entry::<FixedSizeByteString<8>>(&interesting_key).unwrap();
    let writer_entry_id = writer_entry_handle_mut.entry_id();
    let reader_entry_id = reader_entry_handle.entry_id();
    assert_eq!(writer_entry_id, reader_entry_id);
    let entry_id = writer_entry_id;

    g.bench_function("ipc_write_read_complex", |b| {
        b.iter(|| {
            let mut received_count = 0;

            for i in 0..BENCH_MSG_COUNT {
                let msg: FixedSizeByteString<8> = FixedSizeByteString::from_bytes(b"message!").unwrap();

                writer_entry_handle_mut.update_with_copy(msg);
                notifier.notify_with_custom_event_id(entry_id).unwrap();

                if let Some(event_id) = listener.blocking_wait_one().unwrap() {
                    assert!(event_id == entry_id);
                    let received = reader_entry_handle.get();
                    assert_eq!(received, msg);
                    black_box(received);
                    received_count += 1;
                    assert_eq!(received_count, i + 1);
                }
            }
        });
    });

    g.finish();
}

criterion_group!(sync_bench, shm_scalar, shm_complex);
criterion_main!(sync_bench);
