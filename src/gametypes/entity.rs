use bytey::{ByteBufferError, ByteBufferRead, ByteBufferWrite};
use hecs::{EntityRef, World};
use serde::{Deserialize, Serialize};

use crate::{gametypes::*, time_ext::MyInstant};
use core::any::type_name;
use std::ops::{Deref, DerefMut};

#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct Spawn {
    #[derivative(Default(value = "Position::new(10, 10, MapPosition::new(0,0,0))"))]
    pub pos: Position,
    #[derivative(Default(value = "MyInstant::now()"))]
    pub just_spawned: MyInstant,
}

#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct Target {
    pub targettype: EntityType,
    pub targetpos: Position,
    #[derivative(Default(value = "MyInstant::now()"))]
    pub targettimer: MyInstant,
}

#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct KillCount {
    pub count: u32,
    #[derivative(Default(value = "MyInstant::now()"))]
    pub killcounttimer: MyInstant,
}

#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct Vitals {
    #[derivative(Default(value = "[25, 2, 100]"))]
    pub vital: [i32; VITALS_MAX],
    #[derivative(Default(value = "[25, 2, 100]"))]
    pub vitalmax: [i32; VITALS_MAX],
    #[derivative(Default(value = "[0; VITALS_MAX]"))]
    pub vitalbuffs: [i32; VITALS_MAX],
    #[derivative(Default(value = "[0; VITALS_MAX]"))]
    pub regens: [u32; VITALS_MAX],
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Dir(pub u8);

#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct AttackTimer(#[derivative(Default(value = "MyInstant::now()"))] pub MyInstant);

#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct DeathTimer(#[derivative(Default(value = "MyInstant::now()"))] pub MyInstant);

#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct MoveTimer(#[derivative(Default(value = "MyInstant::now()"))] pub MyInstant);

#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct Combat(#[derivative(Default(value = "MyInstant::now()"))] pub MyInstant);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Physical {
    pub damage: u32,
    pub defense: u32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct EntityData(pub [i64; 10]);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Hidden(pub bool);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Stunned(pub bool);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Attacking(pub bool);

#[derive(Derivative, Debug, Copy, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct Level(#[derivative(Default(value = "1"))] pub i32);

#[derive(Derivative, Copy, Debug, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub struct InCombat(#[derivative(Default(value = "false"))] pub bool);

//the World ID stored in our own Wrapper for Packet sending etc.
//This will help ensure we dont try to deal with outdated stuff if we use
// the entire ID rather than just its internal ID.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
pub struct Entity(pub hecs::Entity);

impl Deref for Entity {
    type Target = hecs::Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for Entity {
    fn default() -> Self {
        Self(hecs::Entity::DANGLING)
    }
}

impl DerefMut for Entity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/*
pub etype: EntityType,
    pub mode: NpcMode, //Player is always None
impl Entity {
    pub fn get_id(&self) -> usize {
        self.etype.get_id()
    }

    pub fn reset_target(&mut self) {
        self.targettype = EntityType::None;
        self.targetpos = Position::default();
    }
}*/

impl ByteBufferWrite for Entity {
    fn write_to_buffer(&self, buffer: &mut bytey::ByteBuffer) -> bytey::Result<()> {
        self.0.to_bits().write_to_buffer(buffer)
    }

    fn write_to_buffer_le(&self, buffer: &mut bytey::ByteBuffer) -> bytey::Result<()> {
        self.0.to_bits().write_to_buffer_le(buffer)
    }

    fn write_to_buffer_be(&self, buffer: &mut bytey::ByteBuffer) -> bytey::Result<()> {
        self.0.to_bits().write_to_buffer_be(buffer)
    }
}

impl ByteBufferRead for Entity {
    fn read_from_buffer(buffer: &mut bytey::ByteBuffer) -> bytey::Result<Self>
    where
        Self: Sized,
    {
        Ok(Entity(
            hecs::Entity::from_bits(buffer.read::<u64>()?).ok_or(ByteBufferError::OtherError {
                error: "Bits could nto be converted to hecs Entity. Is your Struct wrong?"
                    .to_owned(),
            })?,
        ))
    }

    fn read_from_buffer_le(buffer: &mut bytey::ByteBuffer) -> bytey::Result<Self>
    where
        Self: Sized,
    {
        Ok(Entity(
            hecs::Entity::from_bits(buffer.read_le::<u64>()?).ok_or(
                ByteBufferError::OtherError {
                    error: "Bits could nto be converted to hecs Entity. Is your Struct wrong?"
                        .to_owned(),
                },
            )?,
        ))
    }

    fn read_from_buffer_be(buffer: &mut bytey::ByteBuffer) -> bytey::Result<Self>
    where
        Self: Sized,
    {
        Ok(Entity(
            hecs::Entity::from_bits(buffer.read_be::<u64>()?).ok_or(
                ByteBufferError::OtherError {
                    error: "Bits could nto be converted to hecs Entity. Is your Struct wrong?"
                        .to_owned(),
                },
            )?,
        ))
    }
}

pub trait WorldExtras {
    fn get_or_default<T>(&self, entity: &Entity) -> T
    where
        T: Default + Send + Sync + Copy + 'static;
    fn cloned_get_or_default<T>(&self, entity: &Entity) -> T
    where
        T: Default + Send + Sync + Copy + 'static;
    fn get_or_panic<T>(&self, entity: &Entity) -> T
    where
        T: Send + Sync + Copy + 'static;
    fn cloned_get_or_panic<T>(&self, entity: &Entity) -> T
    where
        T: Send + Sync + Copy + 'static;
}

pub trait WorldEntityExtras {
    fn get_or_default<T>(&self) -> T
    where
        T: Default + Send + Sync + Copy + 'static;
    fn cloned_get_or_default<T>(&self) -> T
    where
        T: Default + Send + Sync + Copy + 'static;
    fn get_or_panic<T>(&self) -> T
    where
        T: Send + Sync + Copy + 'static;
    fn cloned_get_or_panic<T>(&self) -> T
    where
        T: Send + Sync + Copy + 'static;
}

impl WorldEntityExtras for EntityRef<'_> {
    fn get_or_default<T>(&self) -> T
    where
        T: Default + Send + Sync + Copy + 'static,
    {
        match self.get::<&T>() {
            Some(t) => *t,
            None => T::default(),
        }
    }

    fn cloned_get_or_default<T>(&self) -> T
    where
        T: Default + Send + Sync + Copy + 'static,
    {
        match self.get::<&T>() {
            Some(t) => (*t).clone(),
            None => T::default(),
        }
    }

    fn get_or_panic<T>(&self) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        match self.get::<&T>() {
            Some(t) => *t,
            None => panic!("Component: {} is missing.", type_name::<T>()),
        }
    }

    fn cloned_get_or_panic<T>(&self) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        match self.get::<&T>() {
            Some(t) => (*t).clone(),
            None => panic!("Component: {} is missing.", type_name::<T>()),
        }
    }
}

impl WorldExtras for World {
    fn get_or_default<T>(&self, entity: &Entity) -> T
    where
        T: Default + Send + Sync + Copy + 'static,
    {
        match self.get::<&T>(entity.0) {
            Ok(t) => *t,
            Err(_) => T::default(),
        }
    }

    fn cloned_get_or_default<T>(&self, entity: &Entity) -> T
    where
        T: Default + Send + Sync + Copy + 'static,
    {
        match self.get::<&T>(entity.0) {
            Ok(t) => (*t).clone(),
            Err(_) => T::default(),
        }
    }

    fn get_or_panic<T>(&self, entity: &Entity) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        match self.get::<&T>(entity.0) {
            Ok(t) => *t,
            Err(e) => panic!("Component error: {:?}", e),
        }
    }

    fn cloned_get_or_panic<T>(&self, entity: &Entity) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        match self.get::<&T>(entity.0) {
            Ok(t) => (*t).clone(),
            Err(e) => panic!("Component error: {:?}", e),
        }
    }
}
