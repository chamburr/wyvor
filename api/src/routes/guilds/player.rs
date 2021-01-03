use crate::constants::{
    FILTER_EQUALIZER_BAND_MAX, FILTER_EQUALIZER_BAND_MIN, FILTER_EQUALIZER_GAIN_MAX,
    FILTER_EQUALIZER_GAIN_MIN, FILTER_KARAOKE_BAND_MAX, FILTER_KARAOKE_BAND_MIN,
    FILTER_KARAOKE_LEVEL_MAX, FILTER_KARAOKE_LEVEL_MIN, FILTER_KARAOKE_MONO_LEVEL_MAX,
    FILTER_KARAOKE_MONO_LEVEL_MIN, FILTER_KARAOKE_WIDTH_MAX, FILTER_KARAOKE_WIDTH_MIN,
    FILTER_TIMESCALE_PITCH_MAX, FILTER_TIMESCALE_PITCH_MIN, FILTER_TIMESCALE_RATE_MAX,
    FILTER_TIMESCALE_RATE_MIN, FILTER_TIMESCALE_SPEED_MAX, FILTER_TIMESCALE_SPEED_MIN,
    FILTER_TREMOLO_DEPTH_MAX, FILTER_TREMOLO_DEPTH_MIN, FILTER_TREMOLO_FREQUENCY_MAX,
    FILTER_TREMOLO_FREQUENCY_MIN, FILTER_VIBRATO_DEPTH_MAX, FILTER_VIBRATO_DEPTH_MIN,
    FILTER_VIBRATO_FREQUENCY_MAX, FILTER_VIBRATO_FREQUENCY_MIN, VOLUME_MAX,
};
use crate::db::pubsub::models::{self, Connected};
use crate::db::pubsub::Message;
use crate::db::{PgPool, RedisPool};
use crate::models::{Validate, ValidateExt};
use crate::routes::{ApiResponse, ApiResult};
use crate::utils::auth::User;
use crate::utils::log::{self, LogInfo};
use crate::utils::player::{self, get_player};
use crate::utils::polling;
use crate::utils::queue::{self, Loop};

use actix_web::web::{Data, Json, Path};
use actix_web::{delete, get, patch, post};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use twilight_andesite::model::{Filters, Stop, Update};
use twilight_model::id::GuildId;

#[derive(Debug, Deserialize, Serialize)]
pub struct SimplePlayer {
    pub looping: Option<Loop>,
    pub playing: Option<i32>,
    pub position: Option<u64>,
    pub paused: Option<bool>,
    pub volume: Option<u64>,
    pub filters: Option<Filters>,
}

impl Validate for SimplePlayer {
    fn check(&self) -> ApiResult<()> {
        if let Some(volume) = self.volume {
            (volume as usize).check_max(VOLUME_MAX, "volume")?;
        }

        if let Some(filters) = &self.filters {
            if let Some(equalizer) = &filters.equalizer {
                for band in &equalizer.bands {
                    band.band.check_btw(
                        FILTER_EQUALIZER_BAND_MIN as i64,
                        FILTER_EQUALIZER_BAND_MAX as i64,
                        "equalizer band",
                    )?;
                    band.gain.check_btw(
                        FILTER_EQUALIZER_GAIN_MIN,
                        FILTER_EQUALIZER_GAIN_MAX,
                        "equalizer band gain",
                    )?;
                }
            }
            if let Some(timescale) = &filters.timescale {
                timescale.speed.check_btw(
                    FILTER_TIMESCALE_SPEED_MIN,
                    FILTER_TIMESCALE_SPEED_MAX,
                    "timescale speed",
                )?;
                timescale.pitch.check_btw(
                    FILTER_TIMESCALE_PITCH_MIN,
                    FILTER_TIMESCALE_PITCH_MAX,
                    "timescale pitch",
                )?;
                timescale.rate.check_btw(
                    FILTER_TIMESCALE_RATE_MIN,
                    FILTER_TIMESCALE_RATE_MAX,
                    "timescale rate",
                )?;
            }
            if let Some(tremolo) = &filters.tremolo {
                tremolo.depth.check_btw(
                    FILTER_TREMOLO_DEPTH_MIN,
                    FILTER_TREMOLO_DEPTH_MAX,
                    "tremolo depth",
                )?;
                tremolo.frequency.check_btw(
                    FILTER_TREMOLO_FREQUENCY_MIN,
                    FILTER_TREMOLO_FREQUENCY_MAX,
                    "tremolo frequency",
                )?;
            }
            if let Some(vibrato) = &filters.vibrato {
                vibrato.depth.check_btw(
                    FILTER_VIBRATO_DEPTH_MIN,
                    FILTER_VIBRATO_DEPTH_MAX,
                    "vibrato depth",
                )?;
                vibrato.frequency.check_btw(
                    FILTER_VIBRATO_FREQUENCY_MIN,
                    FILTER_VIBRATO_FREQUENCY_MAX,
                    "vibrato frequency",
                )?;
            }
            if let Some(karaoke) = &filters.karaoke {
                karaoke.level.check_btw(
                    FILTER_KARAOKE_LEVEL_MIN,
                    FILTER_KARAOKE_LEVEL_MAX,
                    "karaoke level",
                )?;
                karaoke.mono_level.check_btw(
                    FILTER_KARAOKE_MONO_LEVEL_MIN,
                    FILTER_KARAOKE_MONO_LEVEL_MAX,
                    "karaoke mono level",
                )?;
                karaoke.filter_band.check_btw(
                    FILTER_KARAOKE_BAND_MIN,
                    FILTER_KARAOKE_BAND_MAX,
                    "karaoke band",
                )?;
                karaoke.filter_width.check_btw(
                    FILTER_KARAOKE_WIDTH_MIN,
                    FILTER_KARAOKE_WIDTH_MAX,
                    "karaoke width",
                )?;
            }
        }

        Ok(())
    }
}

#[get("/{id}/player")]
pub async fn get_guild_player(
    user: User,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;
    user.is_connected(&redis_pool, id, false).await?;

    let player = get_player(&redis_pool, id).await?;

    let mut paused = player.paused;
    let mut position = player.position;
    let looping = queue::get_loop(&redis_pool, id).await?;
    let playing = queue::get_playing(&redis_pool, id).await?;

    if position.is_none() {
        paused = true;
    }

    if position.is_some() && !paused {
        let mut difference = Utc::now().timestamp_millis() - player.time;
        if let Some(timescale) = &player.filters.timescale {
            difference = ((difference as f64) * timescale.speed) as i64;
        }

        position = position.map(|player_position| {
            let mut new_position = player_position + difference;
            if new_position < 0 {
                new_position = 0;
            }
            new_position
        });
    }

    let player = SimplePlayer {
        looping: Some(looping),
        playing: Some(playing),
        position: Some(position.unwrap_or(0) as u64),
        paused: Some(paused),
        volume: Some(player.volume as u64),
        filters: Some(player.filters),
    };

    ApiResponse::ok().data(player).finish()
}

#[post("/{id}/player")]
pub async fn post_guild_player(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;

    let connected: Option<Connected> = Message::get_connected(id, None)
        .send_and_wait(&redis_pool)
        .await?;

    if let Some(connected) = connected {
        if connected.members.len() > 1 {
            return ApiResponse::bad_request()
                .message("The bot is already in another channel.")
                .finish();
        }
    }

    let user_connected: models::Connected = Message::get_connected(id, user.user.id as u64)
        .send_and_wait(&redis_pool)
        .await?
        .ok_or_else(|| {
            ApiResponse::bad_request().message("You need to be connected to a channel.")
        })?;

    Message::set_connected(id, user_connected.channel as u64)
        .send_and_pause(&redis_pool)
        .await?;

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::PlayerAdd(user_connected.channel as u64),
    )
    .await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}

#[patch("/{id}/player")]
pub async fn patch_guild_player(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
    Json(mut new_player): Json<SimplePlayer>,
) -> ApiResult<ApiResponse> {
    user.has_read_guild(&redis_pool, id).await?;
    user.is_connected(&redis_pool, id, true).await?;

    new_player.check()?;

    let player = get_player(&redis_pool, id).await?;

    if let Some(paused) = new_player.paused {
        if !paused && player.position.is_none() {
            if queue::get_playing(&redis_pool, id).await? < 0 {
                queue::next(&redis_pool, id).await?;
            }

            queue::play(&redis_pool, id).await?;

            return ApiResponse::ok().finish();
        }
    }

    user.has_manage_player(&pool, &redis_pool, id).await?;

    if let Some(playing) = new_player.playing {
        if playing >= queue::len(&redis_pool, id).await? as i32 || playing < -1 {
            return ApiResponse::bad_request()
                .message("The requested track to play does not exist.")
                .finish();
        }

        queue::set_playing(&redis_pool, id, playing).await?;

        if playing == -1 {
            player::send(Stop::new(GuildId(id))).await?;
            return ApiResponse::ok().finish();
        }

        queue::play(&redis_pool, id).await?;
    }

    if let Some(looping) = &new_player.looping {
        queue::set_loop(&redis_pool, id, looping).await?;
    }

    player::send(Update::new(
        GuildId(id),
        new_player.paused,
        new_player.position.map(|position| position as i64),
        new_player.volume.map(|volume| volume as i64),
        new_player.filters.clone(),
    ))
    .await?;

    if new_player.position.is_some() {
        new_player.position = player.position.map(|position| position as u64)
    }

    log::register(
        &pool,
        &redis_pool,
        id,
        user,
        LogInfo::PlayerUpdate(new_player),
    )
    .await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}

#[delete("/{id}/player")]
pub async fn delete_guild_player(
    user: User,
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    Path(id): Path<u64>,
) -> ApiResult<ApiResponse> {
    user.has_manage_player(&pool, &redis_pool, id).await?;
    user.is_connected(&redis_pool, id, true).await?;

    let connected: Option<Connected> = Message::get_connected(id, None)
        .send_and_wait(&redis_pool)
        .await?;

    if let Some(connected) = connected {
        log::register(
            &pool,
            &redis_pool,
            id,
            user,
            LogInfo::PlayerRemove(connected.channel as u64),
        )
        .await?;
    }

    Message::set_connected(id, None)
        .send_and_pause(&redis_pool)
        .await?;

    polling::notify(id)?;

    ApiResponse::ok().finish()
}
