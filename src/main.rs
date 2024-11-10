use ::serenity::all::{EventHandler, GatewayIntents, Message};
use ::serenity::prelude::TypeMapKey;
use async_openai::config::OpenAIConfig;
use async_openai::types::ChatCompletionRequestMessage;
use async_openai::Client as OpenAIClient;
use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use std::env;
use std::sync::{Arc, Mutex};

mod oai;

struct Data {
    openai_client: OpenAIClient<OpenAIConfig>,
    ai_context: Arc<Mutex<std::collections::HashMap<String, Vec<ChatCompletionRequestMessage>>>>,
}

impl TypeMapKey for Data {
    type Value = Arc<Data>;
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Arc<Data>, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

// Event handler
struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: serenity::prelude::Context, msg: Message) {
        // are we mentioned?
        // get autorespond channels list from env
        let autorespond_channels: Vec<String> = std::env::var("AUTORESPOND_CHANNELS")
            .unwrap_or("-1302692329400041482".to_string())
            .split(',')
            .map(|s| s.to_string())
            .collect();

        if msg.mentions_user(&ctx.cache.current_user())
            || autorespond_channels.contains(&msg.channel_id.to_string())
                && !msg.author.bot
                && !msg.content.starts_with("~")
        {
            // if we are in certain channels or mentioned
            let cctx = ctx.clone();
            let data = cctx.data.read().await;
            let d = data.get::<Data>().unwrap();
            oai::process_message(msg, ctx, &d.openai_client, &d.ai_context).await;
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let discord_token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
    let openai_key = env::var("OPENAI_API_KEY").expect("Expected OPENAI_API_KEY in environment");
    let openai_base = env::var("OPENAI_BASE_URL").expect("Expected OPENAI_BASE_URL in environment");

    let oai_config: OpenAIConfig = OpenAIConfig::new()
        .with_api_key(openai_key)
        .with_api_base(openai_base);

    let openai_client = OpenAIClient::with_config(oai_config);

    let user_data = Arc::new(Data {
        openai_client,
        ai_context: Arc::new(Mutex::new(std::collections::HashMap::new())),
    });

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let ud_clone = user_data.clone();
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                //poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                // for all guilds we are in
                for guild in ctx.cache.guilds() {
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, guild)
                        .await?;
                }
                Ok(ud_clone)
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(discord_token, intents)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("create client failed");

    {
        let mut data = client.data.write().await;
        data.insert::<Data>(user_data);
    }

    client.start().await.unwrap();
}
