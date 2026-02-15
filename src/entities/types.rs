// src/entities/types.rs
use crate::config::*;
use crate::map::MapHandler;
use super::entity_base::EntityBase;
use super::random_walker::update_random_walker;
use super::stalker::update_stalker;
use super::swarmer::update_swarmer;
use super::sentinel::update_sentinel;
use super::weeping_block::update_weeping_block;
use super::ranger::update_ranger;
use super::tadpole::{update_tadpole, TadpoleState};

pub use super::swarmer::SwarmerState;
pub use super::sentinel::SentinelState;
pub use super::ranger::RangerState;

#[derive(Clone)]
pub enum EntityBehavior {
    RandomWalker {
        change_dir_timer: i32,
    },
    Stalker,
    Swarmer {
        state: SwarmerState,
        wander_timer: i32,
    },
    Sentinel {
        state: SentinelState,
        last_known_pos: Option<(f32, f32)>,
        search_timer: i32,
        patrol_target: (f32, f32),
    },
    WeepingBlock {
        active: bool,
    },
    Ranger {
        state: RangerState,
        charge_timer: i32,
        ideal_range: f32,
        is_firing: bool,
        fire_frame: i32,
    },
    Tadpole {
        state: TadpoleState,
    },
}

pub struct EntityInstance {
    pub base: EntityBase,
    pub behavior: EntityBehavior,
}

impl EntityInstance {
    pub fn new_random_walker(x: f32, y: f32) -> Self {
        Self {
            base: EntityBase::new(x, y, ENTITY_BLUE, 2.0, ENTITY_SIZE),
            behavior: EntityBehavior::RandomWalker { change_dir_timer: 0 },
        }
    }

    pub fn new_stalker(x: f32, y: f32) -> Self {
        Self {
            base: EntityBase::new(x, y, [100.0/255.0, 0.0, 100.0/255.0], 2.5, ENTITY_SIZE),
            behavior: EntityBehavior::Stalker,
        }
    }

    pub fn new_swarmer(x: f32, y: f32) -> Self {
        Self {
            base: EntityBase::new(x, y, [1.0, 1.0, 0.0], 3.5, 8.0),
            behavior: EntityBehavior::Swarmer {
                state: SwarmerState::Passive,
                wander_timer: 0,
            },
        }
    }

    pub fn new_sentinel(x: f32, y: f32) -> Self {
        Self {
            base: EntityBase::new(x, y, [50.0/255.0, 50.0/255.0, 180.0/255.0], 1.2, 25.0),
            behavior: EntityBehavior::Sentinel {
                state: SentinelState::Patrol,
                last_known_pos: None,
                search_timer: 0,
                patrol_target: (x, y),
            },
        }
    }

    pub fn new_weeping_block(x: f32, y: f32) -> Self {
        Self {
            base: EntityBase::new(x, y, [120.0/255.0, 120.0/255.0, 120.0/255.0], 6.0, ENTITY_SIZE),
            behavior: EntityBehavior::WeepingBlock { active: false },
        }
    }

    pub fn new_ranger(x: f32, y: f32) -> Self {
        Self {
            base: EntityBase::new(x, y, [0.0, 1.0, 1.0], 2.0, ENTITY_SIZE),
            behavior: EntityBehavior::Ranger {
                state: RangerState::Positioning,
                charge_timer: 0,
                ideal_range: 300.0,
                is_firing: false,
                fire_frame: 0,
            },
        }
    }

    pub fn new_chaser(x: f32, y: f32) -> Self {
        Self {
            // Tétard : nuances de vert, vitesse modérée, taille standard
            base: EntityBase::new(x, y, [0.2, 0.75, 0.25], 2.2, ENTITY_SIZE),
            behavior: EntityBehavior::Tadpole {
                state: TadpoleState::new(),
            },
        }
    }

    /// Returns a shape ID for 2D map rendering:
    /// 0=circle, 1=triangle, 2=square, 3=diamond, 4=cross, 5=small_circle
    pub fn shape_id(&self) -> u8 {
        match &self.behavior {
            EntityBehavior::RandomWalker { .. } => 0,   // circle
            EntityBehavior::Stalker           => 1,   // triangle
            EntityBehavior::Swarmer { .. }    => 5,   // small circle
            EntityBehavior::Sentinel { .. }   => 2,   // square
            EntityBehavior::WeepingBlock { .. }=> 2,  // square
            EntityBehavior::Ranger { .. }     => 3,   // diamond
            EntityBehavior::Tadpole { .. }    => 6,   // tadpole
        }
    }

    pub fn update(&mut self, player_pos: (f32, f32), player_angle: f32, map: &mut MapHandler, _time_ms: u32, dt: f32) {
        self.base.apply_velocity(map, dt);

        match &mut self.behavior {
            EntityBehavior::RandomWalker { change_dir_timer } => {
                update_random_walker(&mut self.base, change_dir_timer, map, dt);
            }
            EntityBehavior::Stalker => {
                update_stalker(&mut self.base, player_pos, player_angle, map, dt);
            }
            EntityBehavior::Swarmer { state, wander_timer } => {
                update_swarmer(&mut self.base, state, wander_timer, player_pos, map, dt);
            }
            EntityBehavior::Sentinel { state, last_known_pos, search_timer, patrol_target } => {
                update_sentinel(&mut self.base, state, last_known_pos, search_timer, patrol_target, player_pos, map, dt);
            }
            EntityBehavior::WeepingBlock { active } => {
                update_weeping_block(&mut self.base, active, player_pos, player_angle, map, dt);
            }
            EntityBehavior::Ranger { state, charge_timer, ideal_range, is_firing, fire_frame } => {
                update_ranger(&mut self.base, state, charge_timer, ideal_range, is_firing, fire_frame, player_pos, map, dt);
            }
            EntityBehavior::Tadpole { state } => {
                update_tadpole(&mut self.base, state, player_pos, map, dt);
            }
        }
    }
}
