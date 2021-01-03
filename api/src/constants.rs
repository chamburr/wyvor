pub const PUBSUB_CHANNEL: &str = "main";
pub const PLAYER_QUEUE: &str = "player.recv";
pub const PLAYER_SEND_QUEUE: &str = "player.send";

pub const CACHE_DUMP_INTERVAL: usize = 5000;
pub const PLAYER_RECONNECT_WAIT: usize = 2000;

pub const BLACKLIST_REASON_MIN: usize = 1;
pub const BLACKLIST_REASON_MAX: usize = 1000;
pub const VOLUME_MAX: usize = 200;
pub const FILTER_EQUALIZER_BAND_MIN: usize = 0;
pub const FILTER_EQUALIZER_BAND_MAX: usize = 14;
pub const FILTER_EQUALIZER_GAIN_MIN: f64 = -0.25;
pub const FILTER_EQUALIZER_GAIN_MAX: f64 = 1.0;
pub const FILTER_TIMESCALE_SPEED_MIN: f64 = 0.1;
pub const FILTER_TIMESCALE_SPEED_MAX: f64 = 1.9;
pub const FILTER_TIMESCALE_PITCH_MIN: f64 = 0.1;
pub const FILTER_TIMESCALE_PITCH_MAX: f64 = 1.9;
pub const FILTER_TIMESCALE_RATE_MIN: f64 = 0.1;
pub const FILTER_TIMESCALE_RATE_MAX: f64 = 1.9;
pub const FILTER_TREMOLO_DEPTH_MIN: f64 = 0.1;
pub const FILTER_TREMOLO_DEPTH_MAX: f64 = 0.9;
pub const FILTER_TREMOLO_FREQUENCY_MIN: f64 = 0.1;
pub const FILTER_TREMOLO_FREQUENCY_MAX: f64 = 3.9;
pub const FILTER_VIBRATO_DEPTH_MIN: f64 = 0.1;
pub const FILTER_VIBRATO_DEPTH_MAX: f64 = 0.9;
pub const FILTER_VIBRATO_FREQUENCY_MIN: f64 = 0.1;
pub const FILTER_VIBRATO_FREQUENCY_MAX: f64 = 3.9;
pub const FILTER_KARAOKE_LEVEL_MIN: f64 = 0.1;
pub const FILTER_KARAOKE_LEVEL_MAX: f64 = 1.9;
pub const FILTER_KARAOKE_MONO_LEVEL_MIN: f64 = 0.1;
pub const FILTER_KARAOKE_MONO_LEVEL_MAX: f64 = 1.9;
pub const FILTER_KARAOKE_BAND_MIN: f64 = 1.0;
pub const FILTER_KARAOKE_BAND_MAX: f64 = 440.0;
pub const FILTER_KARAOKE_WIDTH_MIN: f64 = 1.0;
pub const FILTER_KARAOKE_WIDTH_MAX: f64 = 199.0;
pub const GUILD_PREFIX_MIN: usize = 1;
pub const GUILD_PREFIX_MAX: usize = 5;
pub const GUILD_QUEUE_MIN: usize = 1;
pub const GUILD_QUEUE_MAX: usize = 5000;
pub const GUILD_ROLES_MAX: usize = 10;
pub const PLAYLIST_MAX: usize = 100;
pub const PLAYLIST_NAME_MIN: usize = 1;
pub const PLAYLIST_NAME_MAX: usize = 50;

pub const COOKIE_NAME: &str = "session";
pub const CALLBACK_PATH: &str = "/callback";

pub const FETCH_USERS_MAX: usize = 100;
pub const FETCH_LOGS_MAX: usize = 100;
pub const FETCH_STAT_DAYS: usize = 7;
pub const FETCH_STAT_TRACKS: usize = 5;
pub const FETCH_STAT_USERS: usize = 5;

pub const BLACKLIST_KEY: &str = "blacklists";
pub const BOT_ADMIN_KEY: &str = "bot_admins";
pub const BOT_OWNER_KEY: &str = "bot_owners";
pub const CSRF_TOKEN_KEY: &str = "csrf_token";
pub const GUILD_KEY: &str = "guild";
pub const GUILD_CONFIG_KEY: &str = "guild_config";
pub const GUILD_PREFIX_KEY: &str = "guild_prefix";
pub const PLAYER_KEY: &str = "player";
pub const PLAYER_STATS_KEY: &str = "player_stats";
pub const QUEUE_KEY: &str = "queue";
pub const QUEUE_LOOP_KEY: &str = "queue_loop";
pub const QUEUE_PLAYING_KEY: &str = "queue_playing";
pub const STATS_KEY: &str = "bot_stats";
pub const STATUS_KEY: &str = "gateway_statuses";
pub const USER_KEY: &str = "user";
pub const USER_GUILDS_KEY: &str = "user_guilds";
pub const USER_TOKEN_KEY: &str = "user_token";

pub const CSRF_TOKEN_KEY_TTL: usize = 300000;
pub const USER_KEY_TTL: usize = 60000;
pub const USER_GUILDS_KEY_TTL: usize = 5000;
pub const USER_TOKEN_KEY_TTL: usize = 60000;

pub fn csrf_token_key(id: &str) -> String {
    format!("{}:{}", CSRF_TOKEN_KEY, id)
}

pub fn guild_key(id: u64) -> String {
    format!("{}:{}", GUILD_KEY, id)
}

pub fn guild_config_key(id: u64) -> String {
    format!("{}:{}", GUILD_CONFIG_KEY, id)
}

pub fn guild_prefix_key(id: u64) -> String {
    format!("{}:{}", GUILD_PREFIX_KEY, id)
}

pub fn player_key(id: u64) -> String {
    format!("{}:{}", PLAYER_KEY, id)
}

pub fn queue_key(id: u64) -> String {
    format!("{}:{}", QUEUE_KEY, id)
}

pub fn queue_loop_key(id: u64) -> String {
    format!("{}:{}", QUEUE_LOOP_KEY, id)
}

pub fn queue_playing_key(id: u64) -> String {
    format!("{}:{}", QUEUE_PLAYING_KEY, id)
}

pub fn user_key(id: u64) -> String {
    format!("{}:{}", USER_KEY, id)
}

pub fn user_guilds_key(id: u64) -> String {
    format!("{}:{}", USER_GUILDS_KEY, id)
}

pub fn user_token_key(id: &str) -> String {
    format!("{}:{}", USER_TOKEN_KEY, id)
}
