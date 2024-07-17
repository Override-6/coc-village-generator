use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::pathfinding::find_route_to_next_building;
use crate::position::Pos;
use crate::scenery::Scenery;
use crate::troop::Troop;
use crate::village::{ComponentId, Village};

#[derive(Default)]
pub struct AttackPlan {
    pub initial_placements: Vec<Troop>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct TroopId(usize);

struct Action {
    kind: ActionKind,
    troop_id: TroopId,
}

enum ActionKind {
    Move(MoveToBuildingAction),
    Attack(AttackAction),
}

struct AttackAction {
    target: ComponentId,
}

struct MoveToBuildingAction {
    path: Vec<Pos>,
    target: ComponentId,
}

pub struct AttackSimulationResult {
    pub village: Village,
    pub troops: Vec<Troop>,
    pub evolution_logs: Vec<EvolutionLog>,
}

pub struct EvolutionLog {
    pub evolution_end_time: f32,
    pub troops_paths: HashMap<TroopId, Vec<Pos>>,
    pub buildings_destroyed: Vec<ComponentId>,
}

pub fn simulate_attack(
    simulation_time_seconds: u32,
    village: &Village,
    attack_plan: &AttackPlan,
) -> AttackSimulationResult {
    let mut village = village.clone();

    let mut troops = attack_plan.initial_placements.clone();

    let mut actions = Vec::new();

    compute_troops_actions(&village, &troops, &mut actions, village.scenery());

    let mut remaining_time = simulation_time_seconds as f32;

    let mut all_evolution_logs = Vec::new();

    while remaining_time > 0.0 {
        let buildings_dps = compute_buildings_dps(&actions, &troops);

        let evolution_time =
            get_shortest_action_completion_time(&actions, &troops, &buildings_dps, &village)
                .min(remaining_time);

        if evolution_time != 0.0 {
            let mut evolution_logs = EvolutionLog {
                evolution_end_time: remaining_time - evolution_time,
                troops_paths: HashMap::default(),
                buildings_destroyed: Vec::default(),
            };

            move_troops(
                &mut actions,
                &mut troops,
                &mut evolution_logs,
                evolution_time,
            );

            for (building_id, dps) in buildings_dps {
                let destroyed = village.damage_component(dps * evolution_time, building_id);
                if destroyed {
                    evolution_logs.buildings_destroyed.push(building_id)
                }
            }

            remaining_time -= evolution_time;
            all_evolution_logs.push(evolution_logs);
        }

        compute_troops_actions(&village, &troops, &mut actions, village.scenery());
    }

    assert_eq!(remaining_time, 0.0);

    AttackSimulationResult {
        village,
        troops,
        evolution_logs: all_evolution_logs,
    }
}

fn compute_buildings_dps(actions: &[Action], troops: &[Troop]) -> HashMap<ComponentId, f32> {
    let mut buildings_damage_per_seconds = HashMap::new();

    for action in actions {
        if let ActionKind::Attack(attack_action) = &action.kind {
            let troop_dps = troops[action.troop_id.0].tpe.damage_per_seconds();
            match buildings_damage_per_seconds.entry(attack_action.target) {
                Entry::Occupied(mut o) => *o.get_mut() += troop_dps,
                Entry::Vacant(v) => {
                    v.insert(troop_dps);
                }
            }
        }
    }

    buildings_damage_per_seconds
}

fn move_troops(
    actions: &mut [Action],
    troops: &mut [Troop],
    logs: &mut EvolutionLog,
    evolution_time: f32,
) {
    for action in actions.iter_mut() {
        if let ActionKind::Move(move_action) = &mut action.kind {
            let troop_id = action.troop_id;
            let walked_path =
                complete_move_action(move_action, &mut troops[troop_id.0], evolution_time);
            logs.troops_paths.insert(troop_id, walked_path);
        }
    }
}

fn compute_troops_actions(
    village: &Village,
    troops: &[Troop],
    actions: &mut Vec<Action>,
    scenery: &Scenery,
) {
    for (troop_id, troop) in troops.iter().enumerate() {
        let troop_id = TroopId(troop_id);

        let troop_action = actions.iter_mut().find(|a| a.troop_id == troop_id);

        match troop_action {
            Some(action) => {
                match &action.kind {
                    ActionKind::Move(move_action) => {
                        if *move_action.path.last().unwrap() != troop.pos {
                            continue; //the action is not completed
                        }

                        // if the completed action is a move, then attack the targeted building
                        *action = Action {
                            kind: ActionKind::Attack(AttackAction {
                                target: move_action.target,
                            }),
                            troop_id,
                        };
                    }

                    ActionKind::Attack(attack_action) => {
                        if !village.is_component_destroyed(attack_action.target) {
                            continue; // keep attacking until the building is destroyed
                        }

                        //if we finished to attack the building (because it got destroyed), move
                        // to another building
                        let move_action = create_move_action(troop, troop_id, village, scenery);
                        // there is no building left, the troop cannot perform further actions, remove its action state

                        match move_action {
                            None => actions.retain(|s| s.troop_id != troop_id),
                            Some(move_action) => *action = move_action,
                        }
                    }
                }
            }
            None => {
                let action = create_move_action(troop, troop_id, village, scenery);

                match action {
                    None => continue,
                    Some(action) => {
                        actions.push(action);
                    }
                }
            }
        }
    }
}

fn create_move_action(
    troop: &Troop,
    troop_id: TroopId,
    village: &Village,
    scenery: &Scenery,
) -> Option<Action> {
    let (path, building_id) = find_route_to_next_building(troop, village, scenery)?;

    if path.is_empty() {
        panic!("Could not find a path to go to building {:?}", building_id);
    }

    Some(Action {
        kind: ActionKind::Move(MoveToBuildingAction {
            path,
            target: building_id,
        }),
        troop_id,
    })
}

/// return the walked path
fn complete_move_action(
    action: &mut MoveToBuildingAction,
    troop: &mut Troop,
    completion_time: f32,
) -> Vec<Pos> {
    let path = &action.path;
    let (troop_new_pos, idx, _) = follow_path_within_time(path, troop, completion_time);

    troop.pos = troop_new_pos;

    // we reach the goal pos of the path
    if troop_new_pos == *path.last().unwrap() {
        return path.clone();
    }

    // else, if we didn't, cut the action's move path to remove all the completed traveling
    let mut new_path = Vec::from(&path[idx..]);
    new_path[0] = troop_new_pos;

    let mut path_walked = Vec::from(&path[..idx - 1]);
    path_walked.push(troop_new_pos);

    action.path = new_path;

    path_walked
}

fn get_shortest_action_completion_time(
    actions: &[Action],
    troops: &[Troop],
    buildings_dps: &HashMap<ComponentId, f32>,
    village: &Village,
) -> f32 {
    let mut smallest = f32::MAX;

    for (building, dps) in buildings_dps {
        let building_lifepoints = village
            .get_component_lifepoints(*building)
            .expect("Cannot damage a building with undefined life points");
        let time_to_destroy = building_lifepoints / dps;
        if smallest > time_to_destroy {
            smallest = time_to_destroy;
        }
    }

    for action in actions {
        match &action.kind {
            ActionKind::Move(move_action) => {
                let completion_time = get_total_completion_time_to_travel(
                    &move_action.path,
                    &troops[action.troop_id.0],
                );
                if smallest > completion_time {
                    smallest = completion_time;
                }
            }
            _ => continue,
        }
    }
    smallest
}

fn get_total_completion_time_to_travel(path: &[Pos], troop: &Troop) -> f32 {
    let Some((first_pos, others)) = path.split_first() else {
        return 0f32;
    };

    let mut current_pos = *first_pos;
    let mut total_distance = 0f32;

    for pos in others {
        let distance_traveled = pos.distance(current_pos);

        total_distance += distance_traveled;

        current_pos = *pos;
    }

    total_distance / troop.tpe.walk_speed()
}

fn follow_path_within_time(path: &[Pos], troop: &Troop, time_limit: f32) -> (Pos, usize, f32) {
    let speed = troop.tpe.walk_speed();

    let mut remaining_distance = speed * time_limit;

    let mut current_pos = troop.pos;

    let mut current_pos_idx = 0;

    for pos in path {
        let distance_traveled = pos.distance(current_pos);

        // we dont have enough remaining distance to entirely go to the next position
        if distance_traveled > remaining_distance {
            // we get the exact position between the current position and the goal to perfectly match the time

            // get the vector between current pos and next pos
            let vector_x = pos.x - current_pos.x;
            let vector_y = pos.y - current_pos.y;

            // get the travelable distance of that vector
            let ratio = remaining_distance / distance_traveled;

            // reduce the vector to the final travelable distance
            let vector_goal_x = vector_x as f32 * ratio;
            let vector_goal_y = vector_y as f32 * ratio;

            let final_position = Pos {
                x: current_pos.x as f32 + vector_goal_x,
                y: current_pos.y as f32 + vector_goal_y,
            };
            return (final_position, current_pos_idx, 0.0);
        }

        remaining_distance -= distance_traveled;

        current_pos = *pos;
        current_pos_idx += 1;
    }

    let total_remaining_time = remaining_distance / speed;

    (current_pos, current_pos_idx, total_remaining_time)
}
