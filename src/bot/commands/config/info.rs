use crate::bot::commands::config::send_settings;
use crate::bot::utils::Utils;
use crate::services::database::DBGuild;
use crate::services::ConnectionPool;
use serenity::framework::standard::macros::command;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::Message;
use serenity::prelude::Context;

#[command]
pub async fn config_info(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = {
        let data = ctx.data.read().await;
        data.get::<ConnectionPool>().unwrap().clone()
    };
    let guild_id = msg.guild_id.unwrap().0 as i64;
    let guild = msg.guild(&ctx).await.unwrap();
    let guild_db: Option<DBGuild> = sqlx::query_as!(
        DBGuild,
        "SELECT * FROM astra.guilds WHERE guild_id = $1",
        guild_id
    )
    .fetch_optional(&pool)
    .await?;

    match guild_db {
        Some(guild_db) => send_settings(&guild_db, &msg, &ctx, &guild).await,
        None => {
            Utils::reply(
                &ctx,
                &msg,
                "Guild not configured please run `>config channel #channel`",
            )
            .await;
        }
    };
    Ok(())
}