use bevy::ecs::schedule::ScheduleLabel;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InGamePreUpdate;
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InGameUpdate;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InGameScorers;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InGameActions;
