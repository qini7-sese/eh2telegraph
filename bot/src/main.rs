#![feature(control_flow_enum)]

use eh2telegraph::{
    collector::Registry,
    config::{self},
    http_proxy::ProxiedClient,
    storage,
    sync::Synchronizer,
    telegraph::Telegraph,
};

use clap::Parser;

use once_cell::sync::OnceCell;
use teloxide::{
    adaptors::DefaultParseMode,
    error_handlers::IgnoringErrorHandler,
    prelude::*,
    types::{AllowedUpdate, ChatPermissions, ParseMode, UpdateKind},
    update_listeners,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use handler::{Command, Handler};

use crate::{
    handler::AdminCommand,
    util::{wrap_endpoint, PrettyChat},
};

mod handler;
mod util;
mod version;

#[derive(Debug, serde::Deserialize)]
pub struct BaseConfig {
    pub bot_token: String,
    pub telegraph: TelegraphConfig,
    #[serde(default)]
    pub admins: Vec<i64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TelegraphConfig {
    pub tokens: Vec<String>,
    pub author_name: Option<String>,
    pub author_url: Option<String>,
}

#[derive(Parser, Debug)]
#[clap(author, version=version::VERSION, about, long_about = "eh2telegraph sync bot")]
struct Args {
    #[clap(short, long, help = "Config file path")]
    config: Option<String>,
}

static PROCESS_MESSAGE_DATE: OnceCell<chrono::DateTime<chrono::Utc>> = OnceCell::new();

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let timer = tracing_subscriber::fmt::time::LocalTime::new(time::macros::format_description!(
        "[month]-[day] [hour]:[minute]:[second]"
    ));
    // We will only process messages from 1 day earlier.
    PROCESS_MESSAGE_DATE
        .set(
            chrono::Utc::now()
                .checked_sub_signed(chrono::Duration::try_days(1).unwrap())
                .expect("illegal current date"),
        )
        .expect("unable to set global date");
    tracing_subscriber::registry()
        .with(fmt::layer().with_timer(timer))
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();
    tracing::info!("initializing...");

    config::init(args.config);
    let base_config: BaseConfig = config::parse("base")
        .expect("unable to parse base config")
        .expect("base config can not be empty");
    let telegraph_config = base_config.telegraph;
    let telegraph =
        Telegraph::new(telegraph_config.tokens).with_proxy(ProxiedClient::new_from_config());

    let registry = Registry::new_from_config();
    #[cfg(debug_assertions)]
    let cache = storage::SimpleMemStorage::default();
    #[cfg(not(debug_assertions))]
    let cache = storage::cloudflare_kv::CFOrMemStorage::new_from_config();
    let mut synchronizer = Synchronizer::new(telegraph, registry, cache);
    if telegraph_config.author_name.is_some() {
        synchronizer =
            synchronizer.with_author(telegraph_config.author_name, telegraph_config.author_url);
    }

    let admins = base_config.admins.into_iter().collect();
    let handler = Box::leak(Box::new(Handler::new(synchronizer, admins))) as &Handler<_>;

    // === Bot related ===
    let command_handler = move |bot: DefaultParseMode<Bot>, message: Message, command: Command| async move {
        handler.respond_cmd(bot, message, command).await
    };
    let admin_command_handler =
        move |bot: DefaultParseMode<Bot>, message: Message, command: AdminCommand| async move {
            handler.respond_admin_cmd(bot, message, command).await
        };
    let text_handler = move |bot: DefaultParseMode<Bot>, message: Message| async move {
        handler.respond_text(bot, message).await
    };
    let caption_handler = move |bot: DefaultParseMode<Bot>, message: Message| async move {
        handler.respond_caption(bot, message).await
    };
    let photo_handler = move |bot: DefaultParseMode<Bot>, message: Message| async move {
        handler.respond_photo(bot, message).await
    };
    let default_handler = move |bot: DefaultParseMode<Bot>, message: Message| async move {
        handler.respond_default(bot, message).await
    };
    let permission_filter = |bot: DefaultParseMode<Bot>, message: Message| async move {
        // If the bot is blocked, we will leave chat and not respond.
        let blocked = message
            .chat
            .permissions()
            .map(|p| !p.contains(ChatPermissions::SEND_MESSAGES))
            .unwrap_or_default();
        if blocked {
            tracing::info!(
                "[permission filter] leave chat {:?}",
                PrettyChat(&message.chat)
            );
            let _ = bot.leave_chat(message.chat.id).await;
            None
        } else {
            Some(message)
        }
    };
    let time_filter = |message: Message| async move {
        // Ignore old message.
        // # Safety:
        // We already set PROCESS_MESSAGE_DATE.
        if &message.date > unsafe { PROCESS_MESSAGE_DATE.get_unchecked() } {
            Some(message)
        } else {
            None
        }
    };

    let bot = Bot::new(base_config.bot_token).parse_mode(ParseMode::MarkdownV2);
    let mut bot_dispatcher = Dispatcher::builder(
        bot.clone(),
        dptree::entry()
            .chain(dptree::filter_map(move |update: Update| {
                match update.kind {
                    UpdateKind::Message(x) | UpdateKind::EditedMessage(x) => Some(x),
                    _ => None,
                }
            }))
            .chain(dptree::filter_map_async(time_filter))
            .chain(dptree::filter_map_async(permission_filter))
            .branch(
                dptree::entry()
                    .chain(dptree::filter(move |message: Message| {
                        handler.admins.contains(&message.chat.id.0)
                    }))
                    .filter_command::<AdminCommand>()
                    .branch(wrap_endpoint(admin_command_handler)),
            )
            .branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .branch(wrap_endpoint(command_handler)),
            )
            .branch(
                dptree::entry()
                    .chain(dptree::filter_map(move |message: Message| {
                        // Ownership mechanism does not allow using map.
                        #[allow(clippy::manual_map)]
                        match message.text() {
                            Some(v) if !v.is_empty() => Some(message),
                            _ => None,
                        }
                    }))
                    .branch(wrap_endpoint(text_handler)),
            )
            .branch(
                dptree::entry()
                    .chain(dptree::filter_map(move |message: Message| {
                        // Ownership mechanism does not allow using map.
                        #[allow(clippy::manual_map)]
                        match message.caption_entities() {
                            Some(v) if !v.is_empty() => Some(message),
                            _ => None,
                        }
                    }))
                    .branch(wrap_endpoint(caption_handler)),
            )
            .branch(
                dptree::entry()
                    .chain(dptree::filter_map(move |message: Message| {
                        // Ownership mechanism does not allow using map.
                        #[allow(clippy::manual_map)]
                        match message.photo() {
                            Some(v) if !v.is_empty() => Some(message),
                            _ => None,
                        }
                    }))
                    .branch(wrap_endpoint(photo_handler)),
            )
            .branch(wrap_endpoint(default_handler)),
    )
    .default_handler(Box::new(|_upd| {
        #[cfg(debug_assertions)]
        tracing::warn!("Unhandled update: {:?}", _upd);
        Box::pin(async {})
    }))
    .error_handler(std::sync::Arc::new(IgnoringErrorHandler))
    .enable_ctrlc_handler()
    .build();
    let bot_listener = update_listeners::Polling::builder(bot)
        .allowed_updates(vec![AllowedUpdate::Message])
        .timeout(std::time::Duration::from_secs(10))
        .build();

    tracing::info!("initializing finished, bot is running");
    bot_dispatcher
        .dispatch_with_listener(
            bot_listener,
            LoggingErrorHandler::with_custom_text("An error from the update listener"),
        )
        .await;
}
