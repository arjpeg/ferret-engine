use std::ops::{Deref, DerefMut};

use anymap::AnyMap;

/// A system is a function that acts on an world, modifying and querying as needed.
pub type System = Box<dyn Fn(&mut World, &mut Resources) + 'static>;

/// A world in which entities along with their associated components live in.
pub struct World(hecs::World);

/// The resources bound to a world.
pub struct Resources(AnyMap);

/// Manages the sequential order of running systems that act on a world.
pub struct Schedule {
    /// The systems to run once during application initialization.
    startup_systems: Vec<System>,
    /// The systems to run during the application's update cycle.
    update_systems: Vec<System>,
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
    pub fn add_startup_system<F: Fn(&mut World, &mut Resources) + 'static>(&mut self, f: F) {
        self.startup_systems.push(Box::new(f));
    }

    /// Adds an update system to the schedule.
    pub fn add_update_system<F: Fn(&mut World, &mut Resources) + 'static>(&mut self, f: F) {
        self.update_systems.push(Box::new(f));
    }

    /// Executes a pass of all systems.
    pub(crate) fn execute(&mut self, world: &mut World, resources: &mut Resources) {
        for system in self.startup_systems.drain(..) {
            system(world, resources);
        }

        for system in &mut self.update_systems {
            system.as_mut()(world, resources);
        }
    }
}

impl World {
    /// Creates a new, empty [`World`].
    pub fn new() -> Self {
        Self(hecs::World::new())
    }
}

impl Resources {
    /// Creates a new, empty set of [`Resources`].
    pub fn new() -> Self {
        Self(AnyMap::new())
    }

    /// Inserts resource, replacing a resource of the same type if it already exists.
    pub fn insert<T: 'static>(&mut self, resource: T) {
        self.0.insert(resource);
    }

    /// Returns an immutable reference to a resource of a given type. Panics if it doesn't exist.
    pub fn get<T: 'static>(&self) -> &T {
        self.0.get::<T>().unwrap()
    }

    /// Returns a mutable reference to a resource of a given type. Panics if it doesn't exist.
    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        self.0.get_mut::<T>().unwrap()
    }
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for World {
    type Target = hecs::World;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
