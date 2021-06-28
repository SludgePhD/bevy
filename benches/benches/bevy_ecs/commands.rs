use bevy::ecs::{
    system::{Command, CommandQueue, Commands},
    world::World,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

criterion_group!(
    benches,
    empty_commands,
    spawn_commands,
    fake_commands,
    zero_sized_commands,
    medium_sized_commands,
    large_sized_commands
);
criterion_main!(benches);

struct A;
struct B;
struct C;

fn empty_commands(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("empty_commands");
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));

    group.bench_function("0_entities", |bencher| {
        let mut world = World::default();
        let mut command_queue = CommandQueue::default();

        bencher.iter(|| {
            command_queue.apply(&mut world);
        });
    });

    group.finish();
}

fn spawn_commands(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("spawn_commands");
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));

    for entity_count in (1..5).map(|i| i * 2 * 1000) {
        group.bench_function(format!("{}_entities", entity_count), |bencher| {
            let mut world = World::default();
            let mut command_queue = CommandQueue::default();

            bencher.iter(|| {
                let mut commands = Commands::new(&mut command_queue, &world);
                for i in 0..entity_count {
                    let mut entity = commands.spawn();

                    if black_box(i % 2 == 0) {
                        entity.insert(A);
                    }

                    if black_box(i % 3 == 0) {
                        entity.insert(B);
                    }

                    if black_box(i % 4 == 0) {
                        entity.insert(C);
                    }

                    if black_box(i % 5 == 0) {
                        entity.despawn();
                    }
                }
                drop(commands);
                command_queue.apply(&mut world);
            });
        });
    }

    group.finish();
}

struct FakeCommandA;
struct FakeCommandB(u64);

impl Command for FakeCommandA {
    fn write(self: Box<Self>, world: &mut World) {
        black_box(self);
        black_box(world);
    }
}

impl Command for FakeCommandB {
    fn write(self: Box<Self>, world: &mut World) {
        black_box(self);
        black_box(world);
    }
}

fn fake_commands(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("fake_commands");
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));

    for command_count in (1..5).map(|i| i * 2 * 1000) {
        group.bench_function(format!("{}_commands", command_count), |bencher| {
            let mut world = World::default();
            let mut command_queue = CommandQueue::default();

            bencher.iter(|| {
                let mut commands = Commands::new(&mut command_queue, &world);
                for i in 0..command_count {
                    if black_box(i % 2 == 0)
                        commands.add(FakeCommandA);
                    } else {
                        commands.add(FakeCommandB(0));
                    }
                }
                drop(commands);
                command_queue.apply(&mut world);
            });
        });
    }

    group.finish();
}

#[derive(Default)]
struct SizedCommand<T: Default + Send + Sync + 'static>(T);

impl<T: Default + Send + Sync + 'static> Command for SizedCommand<T> {
    fn write(self: Box<Self>, world: &mut World) {
        black_box(self);
        black_box(world);
    }
}

struct LargeStruct([u64; 64]);

impl Default for LargeStruct {
    fn default() -> Self {
        Self([0; 64])
    }
}

fn sized_commands_impl<T: Default + Command>(criterion: &mut Criterion) {
    let mut group =
        criterion.benchmark_group(format!("sized_commands_{}_bytes", std::mem::size_of::<T>()));
    group.warm_up_time(std::time::Duration::from_millis(500));
    group.measurement_time(std::time::Duration::from_secs(4));

    for command_count in (1..5).map(|i| i * 2 * 1000) {
        group.bench_function(format!("{}_commands", command_count), |bencher| {
            let mut world = World::default();
            let mut command_queue = CommandQueue::default();

            bencher.iter(|| {
                let mut commands = Commands::new(&mut command_queue, &world);
                for _ in 0..command_count {
                    commands.add(T::default());
                }
                drop(commands);
                command_queue.apply(&mut world);
            });
        });
    }

    group.finish();
}

fn zero_sized_commands(criterion: &mut Criterion) {
    sized_commands_impl::<SizedCommand<()>>(criterion);
}

fn medium_sized_commands(criterion: &mut Criterion) {
    sized_commands_impl::<SizedCommand<(u32, u32, u32)>>(criterion);
}

fn large_sized_commands(criterion: &mut Criterion) {
    sized_commands_impl::<SizedCommand<LargeStruct>>(criterion);
}
