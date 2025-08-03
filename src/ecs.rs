use std::ops::{Deref, DerefMut};

use anymap::AnyMap;
use hecs::DynamicBundle;

/// The main ECS world in which all entities and resources live in.
pub struct World {
    /// The internal hecs-managed ECS world.
    world: hecs::World,
    /// All resources bound to the world.
    resources: AnyMap,
}

/// A system is a function that acts on an world, modifying and querying as needed.
pub type System = Box<dyn Fn(&mut World) + 'static>;

/// Manages the sequential order of running systems that act on a world.
pub struct Schedule {
    /// The systems to run once during application initialization.
    startup_systems: Vec<System>,
    /// The systems to run during the application's update cycle.
    update_systems: Vec<System>,
}

impl World {
    /// Creates a new, empty world with the set of provided resources.
    pub fn new(resources: AnyMap) -> Self {
        Self {
            world: hecs::World::new(),
            resources,
        }
    }

    /// Inserts a resource into the world, replacing it if it already exists.
    pub fn insert_resource<K: 'static>(&mut self, value: K) {
        self.resources.insert::<K>(value);
    }

    /// Attempts to retrieve an immuatable reference to the provided resource type.
    pub fn get_resource<K: 'static>(&self) -> &K {
        self.resources.get::<K>().unwrap()
    }

    /// Attempts to retrieve a muatable reference to the provided resource type.
    pub fn get_resource_mut<K: 'static>(&mut self) -> &mut K {
        self.resources.get_mut::<K>().unwrap()
    }

    /// Spawns a new entity into the world.
    pub fn spawn(&mut self, components: impl DynamicBundle) {
        self.world.spawn(components);
    }
}

impl Schedule {
    /// Creates a new, empty [`Schedule`].
    pub(crate) fn new() -> Self {
        Self {
            startup_systems: Vec::new(),
            update_systems: Vec::new(),
        }
    }

    /// Adds a startup system to the schedule.
    pub fn add_startup_system<F: Fn(&mut World) + 'static>(&mut self, f: F) {
        self.startup_systems.push(Box::new(f));
    }

    /// Adds an update system to the schedule.
    pub fn add_update_system<F: Fn(&mut World) + 'static>(&mut self, f: F) {
        self.update_systems.push(Box::new(f));
    }

    /// Executes a pass of all systems.
    pub(crate) fn execute(&mut self, world: &mut World) {
        for system in self.startup_systems.drain(..) {
            system(world);
        }

        for system in &mut self.update_systems {
            let f = system.as_mut();
            f(world);
        }
    }
}

impl Deref for World {
    type Target = hecs::World;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}
