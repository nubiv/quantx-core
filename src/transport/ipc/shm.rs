use iceoryx2::prelude::*;

const CYCLE_TIME: std::time::Duration = std::time::Duration::from_micros(1);
const INTERESTING_KEY: u32 = 1;

type KeyType = u32;

fn init_creator_iox_bak(service_name: &str) -> Result<(), Box<dyn core::error::Error>> {
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&service_name.try_into()?)
        .blackboard_creator::<KeyType>()
        .add_with_default::<u32>(0)
        .add_with_default::<u32>(INTERESTING_KEY)
        .create()?;

    println!("Blackboard created.\n");

    let event_service = node.service_builder(&service_name.try_into()?).event().open_or_create()?;
    let notifier = event_service.notifier_builder().create()?;

    let writer = service.writer_builder().create()?;

    let entry_handle_mut = writer.entry::<u32>(&0)?;
    let entry_id = entry_handle_mut.entry_id();

    let interesting_entry_handle_mut = writer.entry::<u32>(&INTERESTING_KEY)?;
    let interesting_entry_id = interesting_entry_handle_mut.entry_id();

    // notify with entry id
    let mut counter: u32 = 0;
    while node.wait(CYCLE_TIME).is_ok() {
        counter += 1;
        interesting_entry_handle_mut.update_with_copy(counter);
        notifier.notify_with_custom_event_id(interesting_entry_id)?;
        println!("Trigger event with entry id {}", interesting_entry_id.as_value());

        entry_handle_mut.update_with_copy(2 * counter);
        notifier.notify_with_custom_event_id(entry_id)?;
        println!("Trigger event with entry id {}", entry_id.as_value());
    }

    Ok(())
}

fn init_creator_iox<ServiceKeyType, ServiceValueType>(
    service_name: &str,
) -> Result<
    (
        iceoryx2::port::writer::Writer<ipc::Service, ServiceKeyType>,
        iceoryx2::port::notifier::Notifier<ipc::Service>,
    ),
    Box<dyn core::error::Error>,
>
where
    ServiceKeyType: Sync + Send + Eq + Clone + core::fmt::Debug + core::hash::Hash + iceoryx2::prelude::ZeroCopySend + Default,
    ServiceValueType: Copy + iceoryx2::prelude::ZeroCopySend + Default + 'static,
{
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node
        .service_builder(&service_name.try_into()?)
        .blackboard_creator::<ServiceKeyType>()
        .add_with_default::<ServiceValueType>(ServiceKeyType::default())
        .create()?;

    let event_service = node.service_builder(&service_name.try_into()?).event().open_or_create()?;
    let notifier = event_service.notifier_builder().create()?;

    let writer = service.writer_builder().create()?;

    Ok((writer, notifier))
}

fn init_opener_iox_bak(service_name: &str) -> Result<(), Box<dyn core::error::Error>> {
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node.service_builder(&service_name.try_into()?).blackboard_opener::<KeyType>().open()?;

    let event_service = node.service_builder(&service_name.try_into()?).event().open_or_create()?;
    let listener = event_service.listener_builder().create()?;

    let reader = service.reader_builder().create()?;
    let entry_handle = reader.entry::<u32>(&INTERESTING_KEY)?;

    // wait for entry id
    while node.wait(std::time::Duration::ZERO).is_ok() {
        if let Ok(Some(id)) = listener.timed_wait_one(CYCLE_TIME) {
            if id == entry_handle.entry_id() {
                println!("read: {} for entry id {}", entry_handle.get(), id.as_value());
            }
        }
    }

    // while let Some(_event_id) = listener.blocking_wait_one()? {
    //     println!("event was triggered with id: {:?}", event_id);
    // }

    Ok(())
}

fn init_opener_iox<ServiceKeyType>(
    service_name: &str,
) -> Result<
    (
        iceoryx2::port::reader::Reader<ipc::Service, ServiceKeyType>,
        iceoryx2::port::listener::Listener<ipc::Service>,
    ),
    Box<dyn core::error::Error>,
>
where
    ServiceKeyType: Sync + Send + Eq + Clone + core::fmt::Debug + core::hash::Hash + iceoryx2::prelude::ZeroCopySend + Default,
{
    let node = NodeBuilder::new().create::<ipc::Service>()?;

    let service = node.service_builder(&service_name.try_into()?).blackboard_opener::<ServiceKeyType>().open()?;

    let event_service = node.service_builder(&service_name.try_into()?).event().open_or_create()?;
    let listener = event_service.listener_builder().create()?;

    let reader = service.reader_builder().create()?;

    Ok((reader, listener))
}

pub fn ipc_shm<ServiceKeyType, ServiceValueType>(
    service_name: &str,
) -> (
    iceoryx2::port::writer::Writer<ipc::Service, ServiceKeyType>,
    iceoryx2::port::reader::Reader<ipc::Service, ServiceKeyType>,
    iceoryx2::port::notifier::Notifier<ipc::Service>,
    iceoryx2::port::listener::Listener<ipc::Service>,
)
where
    ServiceKeyType: Sync + Send + Eq + Clone + core::fmt::Debug + core::hash::Hash + iceoryx2::prelude::ZeroCopySend + Default,
    ServiceValueType: Copy + iceoryx2::prelude::ZeroCopySend + Default + 'static,
{
    let (writer, notifier) = init_creator_iox::<ServiceKeyType, ServiceValueType>(service_name).unwrap();
    let (reader, listner) = init_opener_iox::<ServiceKeyType>(service_name).unwrap();

    (writer, reader, notifier, listner)
}
