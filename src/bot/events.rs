use crate::bot::loops::launches_loop;
use crate::bot::utils::Utils;
use crate::extensions::ClientContextExt;


use log::info;
use log::*;
use serenity::{async_trait, model::prelude::*, prelude::*};
use std::sync::Arc;

#[derive(Debug)]
pub struct Handler {
    pub(crate) run_loops: Mutex<bool>,
}

#[allow(unused_must_use)]
#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, _ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache is ready...");
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
        if !is_new {
            return;
        }

        let config = ctx.get_config().await;

        match guild.system_channel_id {
            Some(channel) => Utils::check_msg(
                channel
                    .send_message(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Thanks for adding me!")
                                .description(
                                    "To start you need to setup a launches channel. \
                                    This can be done with `>config channel #launches`. \
                                    I will send launch reminders in that channel",
                                )
                                .footer(|f| {
                                    f.text(&guild.name).icon_url(
                                        &guild.icon_url().unwrap_or_else(|| " ".to_string()),
                                    )
                                })
                        })
                    })
                    .await,
            ),
            None => return,
        }

        let log_channel = Utils::fetch_channel_forced(&ctx, config.log_channel_id)
            .await
            .unwrap();

        let owner_name = match Utils::fetch_user_forced(&ctx, guild.owner_id.0).await {
            Some(owner) => owner.name,
            None => "Owner not found".to_string(),
        };

        Utils::check_msg(
            log_channel
                .id()
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title("Joined Guild")
                            .description(format!(
                                "➤ Member count: **{}**\n ➤ Owner: **{}**",
                                &guild.member_count, owner_name
                            ))
                            .footer(|f| {
                                f.text(&guild.name)
                                    .icon_url(&guild.icon_url().unwrap_or_else(|| " ".to_string()))
                            })
                            .thumbnail(&guild.icon_url().unwrap_or_else(|| " ".to_string()))
                    })
                })
                .await,
        )
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        if reaction.user_id.unwrap() == ctx.cache.current_user_id().await
            || reaction.emoji.to_string() != "🔔"
        {
            return;
        }

        let message = match ctx
            .cache
            .message(reaction.channel_id, reaction.message_id)
            .await
        {
            Some(message) => message,
            None => match reaction.message(&ctx.http).await {
                Ok(message) => message,
                Err(_) => return,
            },
        };

        if message.author.id != ctx.cache.current_user_id().await {
            return;
        }

        let embed = &message.embeds[0];
        let id = &embed.footer.as_ref().unwrap().text;
        let name = &embed.title.as_ref().unwrap();

        let user = match ctx.cache.user(reaction.user_id.unwrap()).await {
            Some(user) => user,
            None => reaction.user(&ctx.http).await.unwrap(),
        };

        let db = ctx.get_db().await;

        reaction.delete(&ctx.http).await;
        let db_user = sqlx::query!(
            "SELECT * FROM astra.reminders WHERE user_id = $1 AND launch_id = $2",
            &(user.id.0 as i64),
            id,
        )
        .fetch_optional(&db.pool)
        .await;

        if let Err(e) = db_user {
            error!("Failed to query, {}", e);
            return;
        }

        let db_user = db_user.unwrap();
        match db_user {
            Some(_) => {
                Utils::check_msg(
                    user.dm(&ctx.http, |m| {
                        m.embed(|e| {
                            e.title("Reminder Removal")
                                .description(format!(
                                    "I will stop reminding you for launch **{}**",
                                    &name
                                ))
                                // .timestamp(chrono::offset::Utc::now())
                                .colour(0xe6e600)
                        })
                    })
                    .await,
                );
                sqlx::query!(
                    "DELETE FROM astra.reminders WHERE user_id = $1 AND launch_id = $2",
                    &(user.id.0 as i64),
                    &id
                )
                .execute(&db.pool)
                .await;
            }
            None => {
                Utils::check_msg(user.dm(&ctx.http, |m| { m
                    .embed(|e| { e
                        .title("Reminder Confirmation")
                        .description(format!("I will remind about launch **{}**. If you want to stop me from reminding you, hit the bell emoji again", &name))
                        // .timestamp(chrono::offset::Utc::now())
                        .colour(0x15c400)
                    })
                }).await);
                sqlx::query!(
                    "INSERT INTO astra.reminders (user_id, launch_id) VALUES ($1, $2)",
                    &(user.id.0 as i64),
                    &id
                )
                .execute(&db.pool)
                .await;
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let perms = Permissions::from_bits(0).unwrap();
        let user = &ready.user;
        ctx.set_presence(
            Some(Activity::listening("new launch announcements")),
            OnlineStatus::Online,
        )
        .await;
        println!(
            "
            Ready as {}
             * Serving {} guilds
             * Invite URL: {}",
            user.tag(),
            ready.guilds.len(),
            user.invite_url(ctx.clone(), perms).await.unwrap(),
        );

        if *self.run_loops.lock().await {
            *self.run_loops.lock().await = false;

            let ctx = Arc::new(ctx);

            let ctx_clone = Arc::clone(&ctx);

            let launches_loop = tokio::spawn(async move { launches_loop(ctx_clone).await });

            let _ = launches_loop.await;
            *self.run_loops.lock().await = false;
        }
    }
}
