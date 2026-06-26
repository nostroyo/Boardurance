//! AI player decision logic for solo-mode opponents.
//!
//! The only in-race decision is the boost card (0-4). [`choose_boost`] implements
//! a single **balanced** profile: advance a sector when it is cheap, avoid dropping
//! a sector, and otherwise conserve high boost cards. It is a pure function of the
//! inputs (no RNG), so solo races are reproducible.
//!
//! All predictions use the **additive** model that lap resolution actually uses
//! (`final = min(engine + body + pilot, sector.max_value) + boost`), exposed here as
//! [`classify_movement`] so the preview endpoints can share the same source of truth.

use crate::domain::{BoostHand, LapCharacteristic, MovementProbability, Sector};
use crate::services::car_validation::ValidatedCarData;

/// Classify the movement a given final performance value produces in a sector,
/// matching the rules in `Race::calculate_movement_for_participant`:
/// `final < min => MoveDown`, `final > max => MoveUp`, otherwise `Stay`.
#[must_use]
pub fn classify_movement(final_value: u32, sector: &Sector) -> MovementProbability {
    if final_value < sector.min_value {
        MovementProbability::MoveDown
    } else if final_value > sector.max_value {
        MovementProbability::MoveUp
    } else {
        MovementProbability::Stay
    }
}

/// The pre-boost performance the AI expects this turn, capped by the sector
/// ceiling. Mirrors the component selection in
/// `Race::calculate_performance_with_car_data`.
fn capped_base_performance(
    car_data: &ValidatedCarData,
    sector: &Sector,
    lap_characteristic: &LapCharacteristic,
) -> u32 {
    let (engine, body, pilot) = match lap_characteristic {
        LapCharacteristic::Straight => (
            u32::from(car_data.engine.straight_value),
            u32::from(car_data.body.straight_value),
            u32::from(car_data.pilot.performance.straight_value),
        ),
        LapCharacteristic::Curve => (
            u32::from(car_data.engine.curve_value),
            u32::from(car_data.body.curve_value),
            u32::from(car_data.pilot.performance.curve_value),
        ),
    };

    std::cmp::min(engine + body + pilot, sector.max_value)
}

/// Choose a boost card (0-4) for an AI participant using the balanced profile.
///
/// 1. If any available card yields `MoveUp`, play the **smallest** such card
///    (advance while conserving high cards).
/// 2. Otherwise, if the car risks `MoveDown` (base below the sector floor), play
///    the smallest available card that reaches `Stay`.
/// 3. Otherwise conserve: play the smallest available card.
///
/// The returned value is always one of `boost_hand.get_available_cards()`.
#[must_use]
pub fn choose_boost(
    car_data: &ValidatedCarData,
    boost_hand: &BoostHand,
    sector: &Sector,
    lap_characteristic: &LapCharacteristic,
) -> u8 {
    let available = boost_hand.get_available_cards();
    // Defensive: a hand always replenishes to 5 cards, so this is unreachable in
    // practice. Fall back to the lowest boost rather than panicking.
    let Some(&smallest) = available.first() else {
        return 0;
    };

    let base = capped_base_performance(car_data, sector, lap_characteristic);

    // 1. Smallest available card that advances a sector.
    if let Some(&boost) = available.iter().find(|&&b| {
        matches!(
            classify_movement(base + u32::from(b), sector),
            MovementProbability::MoveUp
        )
    }) {
        return boost;
    }

    // 2. At risk of dropping a sector: smallest card that avoids MoveDown.
    if base < sector.min_value {
        if let Some(&boost) = available.iter().find(|&&b| {
            !matches!(
                classify_movement(base + u32::from(b), sector),
                MovementProbability::MoveDown
            )
        }) {
            return boost;
        }
    }

    // 3. Nothing to gain or protect: conserve the hand.
    smallest
}

/// An AI's chosen action for a turn: either play a boost card (0-4) or pit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiTurnAction {
    Boost(u8),
    Pit,
}

/// Decide an AI participant's action for the turn, including whether to pit.
///
/// While the pool still has cards, the AI just plays a boost via [`choose_boost`].
/// When the pool is empty (only the free boost 0 remains), a pit stop refills it
/// but costs this lap, so the AI pits only when:
/// - there is at least one more lap to spend the refilled cards on (`laps_remaining > 1`), and
/// - a refilled card could actually change an outcome — either the tyre's strongest
///   card would push the car up a sector, or the car is below the floor (cards could
///   rescue it from dropping).
///
/// (A free boost-0 move can never move up, since base is capped at the sector
/// ceiling, so there's no "already advancing for free" case to exclude.)
///
/// Otherwise the AI takes the free boost-0 move. Pure function of its inputs.
#[must_use]
pub fn decide_ai_action(
    car_data: &ValidatedCarData,
    boost_hand: &BoostHand,
    sector: &Sector,
    lap_characteristic: &LapCharacteristic,
    laps_remaining: u32,
) -> AiTurnAction {
    // Cards available: play one (this also covers the natural free-0 conserve case).
    if boost_hand.cards_remaining > 0 {
        return AiTurnAction::Boost(choose_boost(
            car_data,
            boost_hand,
            sector,
            lap_characteristic,
        ));
    }

    // Pool empty. Weigh a pit (refill, but costs this lap as a free boost-0 move).
    let base = capped_base_performance(car_data, sector, lap_characteristic);
    let strongest = u32::from(
        boost_hand
            .tyre_type
            .initial_pool()
            .into_iter()
            .max()
            .unwrap_or(0),
    );
    let card_would_help = base + strongest > sector.max_value || base < sector.min_value;

    if laps_remaining > 1 && card_would_help {
        AiTurnAction::Pit
    } else {
        AiTurnAction::Boost(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        Body, BodyName, Car, CarName, ComponentRarity, Engine, EngineName, Pilot, PilotClass,
        PilotName, PilotPerformance, PilotRarity, PilotSkills, SectorType,
    };

    /// Build car data with explicit per-axis performance values.
    fn car_with(
        engine_straight: u8,
        engine_curve: u8,
        body_straight: u8,
        body_curve: u8,
        pilot_straight: u8,
        pilot_curve: u8,
    ) -> ValidatedCarData {
        let engine = Engine::new(
            EngineName::parse("AI Engine").unwrap(),
            ComponentRarity::Common,
            engine_straight,
            engine_curve,
        )
        .unwrap();
        let body = Body::new(
            BodyName::parse("AI Body").unwrap(),
            ComponentRarity::Common,
            body_straight,
            body_curve,
        )
        .unwrap();
        let pilot = Pilot::new(
            PilotName::parse("AI Pilot").unwrap(),
            PilotClass::AllRounder,
            PilotRarity::Rookie,
            PilotSkills::new(6, 6, 6, 6).unwrap(),
            PilotPerformance::new(pilot_straight, pilot_curve).unwrap(),
        )
        .unwrap();
        let car = Car::new(CarName::parse("AI Car").unwrap()).unwrap();
        ValidatedCarData {
            car,
            engine,
            body,
            pilot,
        }
    }

    /// Starter-grade car: straight base = 7 + 5 + 8 = 20.
    fn default_car() -> ValidatedCarData {
        car_with(7, 5, 5, 7, 8, 5)
    }

    fn sector(min_value: u32, max_value: u32) -> Sector {
        Sector {
            id: 1,
            name: "Test Sector".to_string(),
            min_value,
            max_value,
            slot_capacity: None,
            sector_type: SectorType::Straight,
        }
    }

    /// A boost hand with the given card values already consumed.
    fn hand_with_used(used: &[u8]) -> BoostHand {
        let mut hand = BoostHand::new();
        for &card in used {
            hand.use_card(card).unwrap();
        }
        hand
    }

    #[test]
    fn picks_smallest_boost_that_moves_up() {
        // base capped at 14 (=max); boost 0 stays. The Medium pool's smallest
        // card is 2 (no value-1 card), so 2 is the smallest that moves up.
        let boost = choose_boost(
            &default_car(),
            &BoostHand::new(),
            &sector(0, 14),
            &LapCharacteristic::Straight,
        );
        assert_eq!(boost, 2);
    }

    #[test]
    fn skips_unavailable_move_up_card() {
        // Both value-2 cards used -> next smallest MoveUp card is 3.
        let boost = choose_boost(
            &default_car(),
            &hand_with_used(&[2, 2]),
            &sector(0, 14),
            &LapCharacteristic::Straight,
        );
        assert_eq!(boost, 3);
    }

    #[test]
    fn avoids_move_down_when_possible() {
        // base capped at 20, below the floor of 22; boost 2 reaches Stay (22).
        let boost = choose_boost(
            &default_car(),
            &BoostHand::new(),
            &sector(22, 30),
            &LapCharacteristic::Straight,
        );
        assert_eq!(boost, 2);
    }

    #[test]
    fn conserves_when_move_down_unavoidable() {
        // Whole pool spent; only the free boost 0 remains, which cannot reach the
        // floor of 22 -> conserve 0.
        let boost = choose_boost(
            &default_car(),
            &hand_with_used(&[2, 2, 3, 3, 4]),
            &sector(22, 30),
            &LapCharacteristic::Straight,
        );
        assert_eq!(boost, 0);
    }

    #[test]
    fn conserves_when_no_move_up_and_safe() {
        // base 20 sits inside [0, 40]; no boost reaches MoveUp -> conserve 0.
        let boost = choose_boost(
            &default_car(),
            &BoostHand::new(),
            &sector(0, 40),
            &LapCharacteristic::Straight,
        );
        assert_eq!(boost, 0);
    }

    #[test]
    fn applies_sector_ceiling_before_boost() {
        // Raw base = 7 + 7 + 9 = 23, capped to 14. Without the cap, boost 0 would
        // already exceed 14 and be chosen; with the cap, a real card is required,
        // and the Medium pool's smallest is 2.
        let strong_car = car_with(7, 7, 7, 7, 9, 9);
        let boost = choose_boost(
            &strong_car,
            &BoostHand::new(),
            &sector(0, 14),
            &LapCharacteristic::Straight,
        );
        assert_eq!(boost, 2);
    }

    #[test]
    fn never_returns_an_unavailable_card() {
        let car = default_car();
        for used in [vec![], vec![2], vec![3], vec![2, 2, 3], vec![2, 2, 3, 3, 4]] {
            let hand = hand_with_used(&used);
            let available = hand.get_available_cards();
            for sec in [sector(0, 14), sector(22, 30), sector(0, 40)] {
                let boost = choose_boost(&car, &hand, &sec, &LapCharacteristic::Straight);
                assert!(
                    available.contains(&boost),
                    "chose unavailable card {boost} from {available:?}"
                );
            }
        }
    }

    #[test]
    fn is_deterministic() {
        let car = default_car();
        let hand = hand_with_used(&[2]);
        let sec = sector(0, 14);
        let a = choose_boost(&car, &hand, &sec, &LapCharacteristic::Straight);
        let b = choose_boost(&car, &hand, &sec, &LapCharacteristic::Straight);
        assert_eq!(a, b);
    }

    #[test]
    fn uses_curve_values_on_curve_laps() {
        // Curve base = engine_curve 5 + body_curve 7 + pilot_curve 5 = 17, capped 14.
        // Smallest Medium-pool card that moves up is 2.
        let boost = choose_boost(
            &default_car(),
            &BoostHand::new(),
            &sector(0, 14),
            &LapCharacteristic::Curve,
        );
        assert_eq!(boost, 2);
    }

    // ---- decide_ai_action (pit strategy) ----
    // default_car straight base = 20 (capped by the sector ceiling).

    #[test]
    fn plays_a_boost_while_cards_remain() {
        // Fresh Medium hand has cards; sector(0,14) caps base to 14, smallest
        // Medium card that moves up is 2.
        let action = decide_ai_action(
            &default_car(),
            &BoostHand::new(),
            &sector(0, 14),
            &LapCharacteristic::Straight,
            5,
        );
        assert_eq!(action, AiTurnAction::Boost(2));
    }

    #[test]
    fn pits_when_empty_and_a_card_would_move_up() {
        // Empty Medium pool; base capped 20, strongest card 4 -> 24 > 22 ceiling.
        let hand = hand_with_used(&[2, 2, 3, 3, 4]);
        assert_eq!(hand.cards_remaining, 0);
        let action = decide_ai_action(
            &default_car(),
            &hand,
            &sector(0, 22),
            &LapCharacteristic::Straight,
            5,
        );
        assert_eq!(action, AiTurnAction::Pit);
    }

    #[test]
    fn pits_when_empty_to_rescue_from_dropping() {
        // Empty pool; base capped 20 is below the floor 22 -> cards could rescue.
        let hand = hand_with_used(&[2, 2, 3, 3, 4]);
        let action = decide_ai_action(
            &default_car(),
            &hand,
            &sector(22, 30),
            &LapCharacteristic::Straight,
            5,
        );
        assert_eq!(action, AiTurnAction::Pit);
    }

    #[test]
    fn does_not_pit_on_the_final_lap() {
        // Same as the move-up case, but no future lap to spend refilled cards on.
        let hand = hand_with_used(&[2, 2, 3, 3, 4]);
        let action = decide_ai_action(
            &default_car(),
            &hand,
            &sector(0, 22),
            &LapCharacteristic::Straight,
            1,
        );
        assert_eq!(action, AiTurnAction::Boost(0));
    }

    #[test]
    fn does_not_pit_when_a_card_would_not_help() {
        // Empty pool; base 20 sits safely inside [0,40] and even +4 can't reach the
        // ceiling, so a refill changes nothing -> take the free boost 0.
        let hand = hand_with_used(&[2, 2, 3, 3, 4]);
        let action = decide_ai_action(
            &default_car(),
            &hand,
            &sector(0, 40),
            &LapCharacteristic::Straight,
            5,
        );
        assert_eq!(action, AiTurnAction::Boost(0));
    }
}
