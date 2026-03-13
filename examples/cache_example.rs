use diself::prelude::*;
use std::env;

struct CacheBot;

#[async_trait]
impl EventHandler for CacheBot {
    async fn on_ready(&self, ctx: &Context, user: User) {
        println!("🤖 {} is ready!", user.tag());
        println!("📦 Current user from cache: {:?}", ctx.cache.current_user());
        println!("📂 Cached channels: {}", ctx.cache.channel_count());
        println!("📂 Cached guilds: {}", ctx.cache.guild_count());
        println!("📂 Cached users: {}", ctx.cache.user_count());
        println!(
            "📂 Cached relationships: {}",
            ctx.cache.relationship_count()
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");

    println!("🦀 Starting Cache Example Bot...\n");

    // Create client with custom cache configuration
    let cache_config = CacheConfig {
        cache_users: true,
        cache_channels: true,
        cache_guilds: true,
        cache_relationships: true,
    };

    let client = Client::new(token, CacheBot)?.with_cache_config(cache_config);

    client.start().await?;

    Ok(())
}
