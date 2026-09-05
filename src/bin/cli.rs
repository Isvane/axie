use toasty_cli::{Config, ToastyCli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = Config::load()?;
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = toasty::Db::builder()
        .models(toasty::models!(axie::*))
        .connect(&database_url)
        .await?;

    let cli = ToastyCli::with_config(db, config);
    cli.parse_and_run().await?;
    Ok(())
}
