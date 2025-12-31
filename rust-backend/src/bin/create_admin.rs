use mongodb::{Client, Database};
use rust_backend::domain::{Car, Email, Password, Pilot, Player, TeamName, UserRole};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get admin credentials from environment or use defaults
    let email = env::var("ADMIN_EMAIL").unwrap_or_else(|_| "admin@racing.game".to_string());
    let password = env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "AdminPass123".to_string());
    let team_name = env::var("ADMIN_TEAM").unwrap_or_else(|_| "Admin Team".to_string());

    println!("Creating admin user...");
    println!("Email: {email}");
    println!("Team: {team_name}");

    // Parse and validate inputs
    let email = Email::parse(&email)?;
    let team_name = TeamName::parse(&team_name)?;
    let password = Password::new(password)?;
    let password_hash = password.hash()?;

    // Create basic car and pilot for the admin user
    let car_name = rust_backend::domain::CarName::parse("Admin Car")?;
    let car = Car::new(car_name, None)?;

    let pilot_name = rust_backend::domain::PilotName::parse("Admin Pilot")?;
    let pilot_skills = rust_backend::domain::PilotSkills {
        reaction_time: 50,
        precision: 50,
        focus: 50,
        stamina: 50,
    };
    let pilot_performance = rust_backend::domain::PilotPerformance {
        straight_value: 50,
        curve_value: 50,
    };
    let pilot = Pilot::new(
        pilot_name,
        rust_backend::domain::PilotClass::AllRounder,
        rust_backend::domain::PilotRarity::Professional,
        pilot_skills,
        pilot_performance,
        None,
    )?;

    // Create admin player
    let mut admin_player = Player::new(email, password_hash, team_name, vec![car], vec![pilot])?;

    // Set admin role
    admin_player.update_role(UserRole::Admin);

    // Connect to MongoDB
    let mongodb_uri = env::var("MONGODB_URI")
        .unwrap_or_else(|_| "mongodb://admin:password123@localhost:27017".to_string());
    let client = Client::with_uri_str(&mongodb_uri).await?;
    let database: Database = client.database("rust_backend");
    let collection = database.collection::<Player>("players");

    // Check if admin user already exists
    let existing_admin = collection
        .find_one(
            mongodb::bson::doc! { "email": admin_player.email.as_ref() },
            None,
        )
        .await?;

    if existing_admin.is_some() {
        println!(
            "❌ Admin user with email {} already exists!",
            admin_player.email.as_ref()
        );
        return Ok(());
    }

    // Insert admin user
    let result = collection.insert_one(&admin_player, None).await?;

    println!("✅ Admin user created successfully!");
    println!("MongoDB ObjectId: {:?}", result.inserted_id);
    println!("UUID: {}", admin_player.uuid);
    println!("\nAdmin Login Credentials:");
    println!("Email: {}", admin_player.email.as_ref());
    println!(
        "Password: {}",
        env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "AdminPass123".to_string())
    );
    println!("Role: {:?}", admin_player.role);
    println!("\nYou can now login to the admin interface at: http://localhost:5173/admin");

    Ok(())
}
