use amethyst::ecs::{ReadExpect, System, WriteExpect};

use std::time::Duration;

use crate::{
    data_resources::{GameEngineState, GameLevelState},
    models::monster_spawn::{Count, SpawnAction, SpawnActions, SpawnType},
    utils::time::GameTimeService,
};

const SECS_PER_LEVEL: u64 = 30;
const MIN_BORDERLINE_INTERVAL_SECS: f32 = 30.0;
const MAX_BORDERLINE_INTERVAL_SECS: f32 = 5.0;

pub struct LevelSystem;

impl LevelSystem {
    pub fn new() -> Self {
        Self
    }
}

impl<'s> System<'s> for LevelSystem {
    type SystemData = (
        GameTimeService<'s>,
        ReadExpect<'s, GameEngineState>,
        WriteExpect<'s, GameLevelState>,
        WriteExpect<'s, SpawnActions>,
    );

    fn run(
        &mut self,
        (game_time_service, game_engine_state, mut game_level_state, mut spawn_actions): Self::SystemData,
    ) {
        if let GameEngineState::Playing = *game_engine_state {
        } else {
            return;
        }

        let now = game_time_service.level_duration();

        if now - game_level_state.spawn_level_started > Duration::from_secs(SECS_PER_LEVEL) {
            game_level_state.spawn_level += 1;
            game_level_state.spawn_level_started = now;
        }

        let borderline_spawn_interval = MIN_BORDERLINE_INTERVAL_SECS
            - (game_level_state.spawn_level as f32 / 7.0).atan() / std::f32::consts::PI
                * 2.0
                * (MAX_BORDERLINE_INTERVAL_SECS - MIN_BORDERLINE_INTERVAL_SECS);
        let borderline_spawn_interval =
            Duration::from_millis((borderline_spawn_interval * 1000.0).round() as u64);
        if now - game_level_state.last_borderline_spawn > borderline_spawn_interval {
            game_level_state.last_borderline_spawn = now;
            spawn_actions.0.push(SpawnAction {
                monsters: Count {
                    entity: "Ghoul".to_owned(),
                    num: 1,
                },
                spawn_type: SpawnType::Borderline,
            });
        }

        let random_spawn_interval = Duration::from_secs(1);
        let monsters_to_spawn = std::cmp::min(game_level_state.spawn_level, 255) as u8;
        if now - game_level_state.last_random_spawn > random_spawn_interval {
            game_level_state.last_random_spawn = now;
            spawn_actions.0.push(SpawnAction {
                monsters: Count {
                    entity: "Ghoul".to_owned(),
                    num: monsters_to_spawn,
                },
                spawn_type: SpawnType::Random,
            });
        }
    }
}
