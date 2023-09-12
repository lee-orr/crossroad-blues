use dexterous_developer::ReloadableAppContents;

pub fn checkpoint_plugin(_app: &mut ReloadableAppContents) {}

#[derive(Clone, Debug)]
pub struct Checkpoint {}
