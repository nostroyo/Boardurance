//! Shared conformance suite for the repository trait implementations.
//!
//! Each function here asserts a single behavior of the trait API (not of any
//! specific implementation) and is called identically from:
//! - `tests/repository_conformance_mock.rs` (runs in `cargo test-fast`, no DB)
//! - `tests/repository_conformance_mongo.rs` (runs in `cargo test-integration`,
//!   needs `MongoDB`)
//!
//! Passing on both proves the `Mock*` repositories are a faithful stand-in for
//! the `Mongo*` ones: same `RepositoryError` variant in the same situations,
//! same success shapes. See `rust-backend/src/repositories/mongo.rs` for the
//! implementations under test and the parity rationale.

use rust_backend::domain::{
    Body, BodyName, Car, CarName, ComponentRarity, Email, Engine, EngineName, HashedPassword,
    LapAction, Pilot, PilotClass, PilotName, PilotPerformance, PilotRarity, PilotSkills, Player,
    Race, RaceStatus, Sector, SectorType, TeamName, Track,
};
use rust_backend::repositories::{
    PlayerRepository, RaceRepository, RepositoryError, SessionRepository,
};
use rust_backend::services::car_validation::ValidatedCarData;
use rust_backend::services::session::Session;
use uuid::Uuid;

/// Build a minimal, valid player with no assets. Each call gets a unique
/// random email/uuid so tests run against a shared (Mongo) database don't
/// collide with each other or with a previous run.
pub fn make_player() -> Player {
    let unique = Uuid::new_v4();
    let email = Email::parse(&format!("conformance-{unique}@example.com")).unwrap();
    let team_name = TeamName::parse(&format!("Team {unique}")).unwrap();
    let password_hash = HashedPassword::from_hash("test_hash".to_string());
    Player::new(email, password_hash, team_name, Vec::new(), Vec::new()).unwrap()
}

/// Build a complete, race-ready car/engine/body/pilot bundle plus the pilot's
/// `uuid` (needed by `RaceRepository::join_race` and friends).
pub fn make_validated_car_data() -> ValidatedCarData {
    let engine = Engine::new(
        EngineName::parse("Conformance Engine").unwrap(),
        ComponentRarity::Common,
        7,
        5,
    )
    .unwrap();
    let body = Body::new(
        BodyName::parse("Conformance Body").unwrap(),
        ComponentRarity::Common,
        5,
        7,
    )
    .unwrap();
    let pilot = Pilot::new(
        PilotName::parse("Conformance Pilot").unwrap(),
        PilotClass::AllRounder,
        PilotRarity::Rookie,
        PilotSkills::new(6, 6, 6, 6).unwrap(),
        PilotPerformance::new(8, 5).unwrap(),
    )
    .unwrap();
    let car = Car::new(CarName::parse("Conformance Car").unwrap()).unwrap();
    ValidatedCarData {
        car,
        engine,
        body,
        pilot,
    }
}

/// Build a small track suitable for a race that can actually be joined and
/// started (matches the shape used elsewhere in the domain tests).
pub fn make_track() -> Track {
    let sectors = vec![
        Sector {
            id: 0,
            name: "Start".to_string(),
            min_value: 0,
            max_value: 10,
            slot_capacity: None,
            sector_type: SectorType::Start,
        },
        Sector {
            id: 1,
            name: "Straight".to_string(),
            min_value: 5,
            max_value: 14,
            slot_capacity: None,
            sector_type: SectorType::Straight,
        },
        Sector {
            id: 2,
            name: "Finish".to_string(),
            min_value: 8,
            max_value: 16,
            slot_capacity: None,
            sector_type: SectorType::Finish,
        },
    ];
    Track::new("Conformance Track".to_string(), sectors).unwrap()
}

/// Build a `Waiting` race with the given `total_laps`, ready to be joined.
pub fn make_race(total_laps: u32) -> Race {
    Race::new("Conformance Race".to_string(), make_track(), total_laps)
}

// ===================== PlayerRepository conformance =====================

/// Create then find the same player back by email and by uuid.
pub async fn player_create_and_find_round_trips(repo: &dyn PlayerRepository) {
    let player = make_player();

    let created = repo.create(&player).await.expect("create should succeed");
    assert_eq!(created.uuid, player.uuid);

    let by_uuid = repo
        .find_by_uuid(player.uuid)
        .await
        .expect("find_by_uuid should not error")
        .expect("player should be found by uuid");
    assert_eq!(by_uuid.uuid, player.uuid);

    let by_email = repo
        .find_by_email(player.email.as_ref())
        .await
        .expect("find_by_email should not error")
        .expect("player should be found by email");
    assert_eq!(by_email.uuid, player.uuid);
}

/// A missing player resolves to `Ok(None)`, not an error — for both lookup
/// methods.
pub async fn player_find_missing_returns_none(repo: &dyn PlayerRepository) {
    let missing_uuid = Uuid::new_v4();
    assert!(repo
        .find_by_uuid(missing_uuid)
        .await
        .expect("lookup should not error")
        .is_none());

    let missing_email = format!("no-such-{missing_uuid}@example.com");
    assert!(repo
        .find_by_email(&missing_email)
        .await
        .expect("lookup should not error")
        .is_none());
}

/// Creating a second player with the same email must fail with `Conflict`,
/// not succeed or fail with a different variant.
pub async fn player_duplicate_email_is_conflict(repo: &dyn PlayerRepository) {
    let player = make_player();
    repo.create(&player).await.expect("first create succeeds");

    // Same email, different uuid/team — only the email needs to collide.
    let team_name = TeamName::parse("Another Team").unwrap();
    let mut duplicate = Player::new(
        player.email.clone(),
        player.password_hash.clone(),
        team_name,
        vec![],
        vec![],
    )
    .unwrap();
    duplicate.uuid = Uuid::new_v4();

    let err = repo
        .create(&duplicate)
        .await
        .expect_err("duplicate email must be rejected");
    assert!(
        matches!(err, RepositoryError::Conflict(_)),
        "expected Conflict, got {err:?}"
    );
}

/// `update_team_name_by_uuid` persists the new name and is observable via a
/// fresh read (proves it round-trips through storage, not just an in-memory
/// mutation of the caller's copy).
pub async fn player_update_team_name_round_trips(repo: &dyn PlayerRepository) {
    let player = make_player();
    repo.create(&player).await.unwrap();

    let new_name = TeamName::parse("Renamed Team").unwrap();
    let updated = repo
        .update_team_name_by_uuid(player.uuid, new_name.clone())
        .await
        .expect("update should not error")
        .expect("player should exist");
    assert_eq!(updated.team_name.as_ref(), new_name.as_ref());

    let reloaded = repo
        .find_by_uuid(player.uuid)
        .await
        .unwrap()
        .expect("player should still exist");
    assert_eq!(reloaded.team_name.as_ref(), new_name.as_ref());
}

/// Mutating a player that doesn't exist returns `Ok(None)`.
pub async fn player_update_missing_returns_none(repo: &dyn PlayerRepository) {
    let missing_uuid = Uuid::new_v4();
    let new_name = TeamName::parse("Nobodys Team").unwrap();
    let result = repo
        .update_team_name_by_uuid(missing_uuid, new_name)
        .await
        .expect("should not error");
    assert!(result.is_none());
}

/// Add then remove a car by uuid; both mutations round-trip through storage.
pub async fn player_add_and_remove_car_round_trips(repo: &dyn PlayerRepository) {
    let player = make_player();
    repo.create(&player).await.unwrap();

    let car = Car::new(CarName::parse("Added Car").unwrap()).unwrap();
    let car_uuid = car.uuid;

    let after_add = repo
        .add_car_by_uuid(player.uuid, car)
        .await
        .unwrap()
        .expect("player should exist");
    assert!(after_add.cars.iter().any(|c| c.uuid == car_uuid));

    let after_remove = repo
        .remove_car_by_uuid(player.uuid, car_uuid)
        .await
        .unwrap()
        .expect("player should exist");
    assert!(!after_remove.cars.iter().any(|c| c.uuid == car_uuid));
}

/// Deleting an existing player returns `true` and the player is gone;
/// deleting again returns `false`.
pub async fn player_delete_by_uuid_works(repo: &dyn PlayerRepository) {
    let player = make_player();
    repo.create(&player).await.unwrap();

    assert!(repo.delete_by_uuid(player.uuid).await.unwrap());
    assert!(repo.find_by_uuid(player.uuid).await.unwrap().is_none());
    assert!(!repo.delete_by_uuid(player.uuid).await.unwrap());
}

// ===================== RaceRepository conformance =====================

/// Create then find the same race back by uuid.
pub async fn race_create_and_find_round_trips(repo: &dyn RaceRepository) {
    let race = make_race(3);
    let created = repo.create(&race).await.expect("create should succeed");
    assert_eq!(created.uuid, race.uuid);

    let found = repo
        .find_by_uuid(race.uuid)
        .await
        .expect("find should not error")
        .expect("race should be found");
    assert_eq!(found.uuid, race.uuid);
    assert_eq!(found.status, RaceStatus::Waiting);
}

/// A missing race resolves to `Ok(None)`.
pub async fn race_find_missing_returns_none(repo: &dyn RaceRepository) {
    let missing = repo.find_by_uuid(Uuid::new_v4()).await.unwrap();
    assert!(missing.is_none());
}

/// Joining a non-existent race is `NotFound`.
pub async fn race_join_missing_race_is_not_found(repo: &dyn RaceRepository) {
    let car_data = make_validated_car_data();
    let err = repo
        .join_race(Uuid::new_v4(), car_data.pilot.uuid, &car_data)
        .await
        .expect_err("joining a missing race must fail");
    assert!(matches!(err, RepositoryError::NotFound), "got {err:?}");
}

/// The same pilot joining the same race twice is `Conflict` on the second
/// attempt (mirrors the domain's duplicate-participant guard).
pub async fn race_duplicate_join_is_conflict(repo: &dyn RaceRepository) {
    let race = make_race(3);
    repo.create(&race).await.unwrap();
    let car_data = make_validated_car_data();

    repo.join_race(race.uuid, car_data.pilot.uuid, &car_data)
        .await
        .expect("first join should succeed");

    let err = repo
        .join_race(race.uuid, car_data.pilot.uuid, &car_data)
        .await
        .expect_err("second join by the same pilot must fail");
    assert!(matches!(err, RepositoryError::Conflict(_)), "got {err:?}");
}

/// Joining a race that isn't `Waiting` is rejected as `Validation` (the
/// domain's `add_participant` status guard, surfaced unchanged through the
/// repository).
pub async fn race_join_non_waiting_race_is_validation_error(repo: &dyn RaceRepository) {
    let mut race = make_race(3);
    race.status = RaceStatus::Finished;
    repo.create(&race).await.unwrap();

    let car_data = make_validated_car_data();
    let err = repo
        .join_race(race.uuid, car_data.pilot.uuid, &car_data)
        .await
        .expect_err("joining a finished race must fail");
    assert!(matches!(err, RepositoryError::Validation(_)), "got {err:?}");
}

/// Full race-turn flow: join, start, submit a turn via
/// `process_turn_actions`, then reload the race from the repository and
/// confirm the mutation (lap advance) actually persisted — not just returned
/// to the caller.
pub async fn race_turn_processing_persists(repo: &dyn RaceRepository) {
    let mut race = make_race(2);
    let car_data = make_validated_car_data();
    race.add_participant(car_data.pilot.uuid, car_data.car.uuid, car_data.pilot.uuid)
        .expect("adding the sole participant directly should succeed");
    race.start_race()
        .expect("race with a participant should start");
    repo.create(&race).await.unwrap();

    let actions = vec![LapAction {
        player_uuid: car_data.pilot.uuid,
        boost_value: 2,
    }];

    let (_, status_after) = repo
        .process_turn_actions(race.uuid, car_data.pilot.uuid, actions)
        .await
        .expect("processing the turn should not error")
        .expect("race should exist");

    let reloaded = repo
        .find_by_uuid(race.uuid)
        .await
        .unwrap()
        .expect("race should still exist");
    assert_eq!(
        reloaded.status, status_after,
        "the status returned by process_turn_actions must match what's persisted"
    );
    assert!(
        reloaded.turns_taken >= 1,
        "the persisted race must reflect the processed turn"
    );
}

/// Processing a turn for a race that hasn't started is a `Validation` error
/// (mirrors `Race::process_lap`'s status guard).
pub async fn race_turn_processing_before_start_is_validation_error(repo: &dyn RaceRepository) {
    let race = make_race(2);
    repo.create(&race).await.unwrap();

    let err = repo
        .process_turn_actions(race.uuid, Uuid::new_v4(), vec![])
        .await
        .expect_err("processing a turn on a Waiting race must fail");
    assert!(matches!(err, RepositoryError::Validation(_)), "got {err:?}");
}

/// `get_races_by_status` returns exactly the races matching the requested
/// status.
pub async fn race_get_by_status_filters_correctly(repo: &dyn RaceRepository) {
    let waiting = make_race(1);
    let mut finished = make_race(1);
    finished.status = RaceStatus::Finished;

    repo.create(&waiting).await.unwrap();
    repo.create(&finished).await.unwrap();

    let waiting_races = repo.get_races_by_status(RaceStatus::Waiting).await.unwrap();
    assert!(waiting_races.iter().any(|r| r.uuid == waiting.uuid));
    assert!(!waiting_races.iter().any(|r| r.uuid == finished.uuid));

    let finished_races = repo
        .get_races_by_status(RaceStatus::Finished)
        .await
        .unwrap();
    assert!(finished_races.iter().any(|r| r.uuid == finished.uuid));
    assert!(!finished_races.iter().any(|r| r.uuid == waiting.uuid));
}

// ===================== SessionRepository conformance =====================

fn make_session(user_uuid: Uuid) -> Session {
    let now = chrono::Utc::now();
    Session {
        id: None,
        user_uuid,
        token: format!("conformance-token-{}", Uuid::new_v4()),
        created_at: now,
        last_activity: now,
        expires_at: now + chrono::Duration::hours(1),
        ip_address: None,
        user_agent: None,
        is_active: true,
        updated_at: now,
    }
}

/// Create then find the same session back by token.
pub async fn session_create_and_find_round_trips(repo: &dyn SessionRepository) {
    let user_uuid = Uuid::new_v4();
    let session = make_session(user_uuid);
    repo.create(&session).await.expect("create should succeed");

    let found = repo
        .find_by_token(&session.token)
        .await
        .expect("find should not error")
        .expect("session should be found");
    assert_eq!(found.user_uuid, user_uuid);
    assert!(found.is_active);
}

/// A missing token resolves to `Ok(None)`.
pub async fn session_find_missing_returns_none(repo: &dyn SessionRepository) {
    let missing = repo
        .find_by_token(&format!("missing-{}", Uuid::new_v4()))
        .await
        .unwrap();
    assert!(missing.is_none());
}

/// An expired session is not returned by `find_by_token`, even though the
/// document still exists (matches the mock's active+not-expired filter).
pub async fn session_expired_is_not_found(repo: &dyn SessionRepository) {
    let user_uuid = Uuid::new_v4();
    let mut session = make_session(user_uuid);
    session.expires_at = chrono::Utc::now() - chrono::Duration::hours(1);
    repo.create(&session).await.unwrap();

    let found = repo.find_by_token(&session.token).await.unwrap();
    assert!(found.is_none(), "an expired session must not be returned");
}

/// `deactivate` flips `is_active` such that the session is no longer
/// returned by `find_by_token`.
pub async fn session_deactivate_hides_session(repo: &dyn SessionRepository) {
    let user_uuid = Uuid::new_v4();
    let session = make_session(user_uuid);
    repo.create(&session).await.unwrap();

    repo.deactivate(&session.token)
        .await
        .expect("deactivate should not error");

    let found = repo.find_by_token(&session.token).await.unwrap();
    assert!(
        found.is_none(),
        "a deactivated session must not be returned"
    );
}

/// `deactivate_all_for_user` deactivates every active session for that user
/// and leaves other users' sessions untouched.
pub async fn session_deactivate_all_for_user_is_scoped(repo: &dyn SessionRepository) {
    let user_a = Uuid::new_v4();
    let user_b = Uuid::new_v4();
    let session_a1 = make_session(user_a);
    let session_a2 = make_session(user_a);
    let session_b = make_session(user_b);

    repo.create(&session_a1).await.unwrap();
    repo.create(&session_a2).await.unwrap();
    repo.create(&session_b).await.unwrap();

    repo.deactivate_all_for_user(user_a)
        .await
        .expect("should not error");

    assert!(repo
        .find_by_token(&session_a1.token)
        .await
        .unwrap()
        .is_none());
    assert!(repo
        .find_by_token(&session_a2.token)
        .await
        .unwrap()
        .is_none());
    assert!(
        repo.find_by_token(&session_b.token)
            .await
            .unwrap()
            .is_some(),
        "another user's session must be unaffected"
    );
}

/// `cleanup_expired` removes only expired sessions and reports how many were
/// removed.
pub async fn session_cleanup_expired_removes_only_expired(repo: &dyn SessionRepository) {
    let user_uuid = Uuid::new_v4();
    let mut expired = make_session(user_uuid);
    expired.expires_at = chrono::Utc::now() - chrono::Duration::hours(1);
    let active = make_session(user_uuid);

    repo.create(&expired).await.unwrap();
    repo.create(&active).await.unwrap();

    let removed = repo
        .cleanup_expired(chrono::Utc::now())
        .await
        .expect("cleanup should not error");
    assert!(
        removed >= 1,
        "the expired session must be counted as removed"
    );

    // The still-active session must survive cleanup (re-fetch bypassing the
    // is_active/expiry filter isn't available, so assert indirectly: it's
    // still findable via find_by_token, which the expired one is not).
    assert!(repo.find_by_token(&active.token).await.unwrap().is_some());
}

/// `count_active_for_user` only counts active, non-expired sessions for that
/// specific user.
pub async fn session_count_active_for_user_is_accurate(repo: &dyn SessionRepository) {
    let user_uuid = Uuid::new_v4();
    let other_user = Uuid::new_v4();

    let active1 = make_session(user_uuid);
    let active2 = make_session(user_uuid);
    let mut expired = make_session(user_uuid);
    expired.expires_at = chrono::Utc::now() - chrono::Duration::hours(1);
    let other = make_session(other_user);

    repo.create(&active1).await.unwrap();
    repo.create(&active2).await.unwrap();
    repo.create(&expired).await.unwrap();
    repo.create(&other).await.unwrap();

    let count = repo.count_active_for_user(user_uuid).await.unwrap();
    assert_eq!(
        count, 2,
        "only the two active, non-expired sessions for this user should count"
    );
}
