use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    println!("🌱 Seeding admin user...");

    let pool = PgPool::connect(&database_url).await?;

    // Check if admin already exists
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = 'admin@example.com')"
    )
    .fetch_one(&pool)
    .await?;

    if exists {
        println!("⚠️  Admin user already exists!");
        return Ok(());
    }

    // Hash the default password
    let password_hash = password_auth::generate_hash("admin123456");

    // Create admin user
    sqlx::query(
        r#"
        INSERT INTO users (email, password_hash, first_name, last_name, role, is_active, email_verified)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#
    )
    .bind("admin@example.com")
    .bind(&password_hash)
    .bind("Admin")
    .bind("User")
    .bind("admin")
    .bind(true)
    .bind(true)
    .execute(&pool)
    .await?;

    println!("✅ Admin user created successfully!");
    println!();
    println!("📧 Email: admin@example.com");
    println!("🔑 Password: admin123456");
    println!();
    println!("⚠️  IMPORTANT: Change this password immediately after first login!");

    Ok(())
}
