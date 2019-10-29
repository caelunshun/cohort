#[macro_use]
extern crate cohort;

use legion::system::StageExecutor;
use legion::world::Universe;

struct TestResource1(usize);
#[derive(Debug, PartialEq)]
struct TestResource2(usize);

#[system]
fn test_system(test1: &TestResource1, test2: &mut TestResource2) {
    assert_eq!(test1.0, 1);
    test2.0 += 1;
}

#[test]
fn basic() {
    let universe = Universe::new();
    let mut world = universe.create_world();
    world.resources.insert(TestResource1(1));
    world.resources.insert(TestResource2(4));

    let test_system = test_system();
    let mut systems = [test_system];
    let mut scheduler = StageExecutor::new(&mut systems);

    scheduler.execute(&world);

    assert_eq!(world.resources.get::<TestResource2>().unwrap().0, 5);
}
