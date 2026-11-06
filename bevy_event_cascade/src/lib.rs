use std::{
    any::type_name,
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
};

use bevy_app::App;
use bevy_ecs::{
    component::Component,
    entity::Entity,
    error::Result,
    event::EntityEvent,
    observer::On,
    resource::Resource,
    system::{Commands, Query, Res},
};

extern crate self as bevy_event_cascade;

type Phantom<T> = PhantomData<fn() -> T>;

#[derive(Component)]
pub struct StageProgress<S> {
    count: usize,
    _marker: Phantom<S>,
}

impl<S> Default for StageProgress<S> {
    fn default() -> Self {
        Self {
            count: 0,
            _marker: PhantomData,
        }
    }
}

pub trait InStage {
    type Stage;
}

#[derive(EntityEvent)]
pub struct StageCompleted<S> {
    entity: Entity,
    _marker: Phantom<S>,
}

impl<S> StageCompleted<S> {
    fn new(entity: Entity) -> Self {
        Self {
            entity,
            _marker: PhantomData,
        }
    }
}

pub trait EventCascadeAppExt {
    fn register_event<E>(&mut self) -> &mut Self
    where
        E: EntityEvent + InStage;
}

impl EventCascadeAppExt for App {
    fn register_event<E>(&mut self) -> &mut Self
    where
        E: EntityEvent + InStage,
    {
        self.world_mut()
            .init_resource::<DependencyCounter<E::Stage>>();

        self.world_mut()
            .resource_mut::<DependencyCounter<E::Stage>>()
            .total += 1;

        self.add_observer(phase_part_complete::<E>)
    }
}

#[derive(Resource)]
struct DependencyCounter<S> {
    total: usize,
    _marker: Phantom<S>,
}

impl<S> Default for DependencyCounter<S> {
    fn default() -> Self {
        Self {
            total: 0,
            _marker: PhantomData,
        }
    }
}

fn phase_part_complete<E>(
    event: On<E>,
    mut progress: Query<&mut StageProgress<E::Stage>>,
    counter: Res<DependencyCounter<E::Stage>>,
    mut commands: Commands,
) -> Result
where
    E: EntityEvent + InStage,
{
    let mut progress = progress
        .get_mut(event.event_target())
        .map_err(|_| MissingStageProgressComponent::<E::Stage>::new(event.event_target()))?;

    progress.count += 1;
    if progress.count == counter.total {
        commands
            .entity(event.event_target())
            .remove::<StageProgress<E::Stage>>();

        commands.trigger(StageCompleted::<E::Stage>::new(event.event_target()));
    }

    Ok(())
}

pub struct MissingStageProgressComponent<S> {
    entity: Entity,
    _marker: Phantom<S>,
}

impl<S> MissingStageProgressComponent<S> {
    fn new(entity: Entity) -> Self {
        Self {
            entity,
            _marker: PhantomData,
        }
    }
}

impl<S> Debug for MissingStageProgressComponent<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MissingStageProgressComponent<{}>", type_name::<S>())
    }
}

impl<S> Display for MissingStageProgressComponent<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let stage = type_name::<S>();
        write!(
            f,
            "an event in stage `{stage}` was triggered for entity {entity}, \
             but the entity has no `StageProgress<{stage}>` component.\n\
             Possible causes:\n  \
             (1) the component was never inserted. Insert \
             `StageProgress::<{stage}>::default()` on the entity before \
             triggering the first event of the cascade.\n  \
             (2) the entity was despawned while the cascade was still running.\n  \
             (3) the stage already completed for this entity, which removes the \
             component. Insert it again before starting another run, and make \
             sure no event of the stage is triggered more than once.",
            entity = self.entity
        )
    }
}

impl<S> std::error::Error for MissingStageProgressComponent<S> {}
