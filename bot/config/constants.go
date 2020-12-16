package config

import (
	"time"
)

const (
	DiscordMaxRestRetries = 3
	DiscordMaxCCRequests  = 50

	HttpMaxIdleConns          = 100
	HttpMaxIdleConnsPerHost   = 100
	HttpDialKeepAlive         = 10 * time.Second
	HttpDialTimeout           = 10 * time.Second
	HttpIdleConnTimeout       = 90 * time.Second
	HttpTlsHandshakeTimeout   = 5 * time.Second
	HttpExpectContinueTimeout = 1 * time.Second
	HttpLogDurationThreshold  = 5 * time.Second

	StateCacheExpiry = 1 * time.Hour

	ApiTimeout = 5 * time.Second

	RabbitQueueReceive = "gateway"
	RabbitQueueSend    = "gateway.send"

	RedisMaxConnections      = 10
	RedisPubsubChannel       = "main"
	RedisGuildsCacheExpiry   = 15 * time.Second
	RedisGuildsFlushInterval = 5 * time.Second
	RedisStatsFlushInterval  = 5 * time.Second
	RedisAdminsFlushInterval = 5 * time.Second

	MetricsFlushInterval = 5 * time.Second

	VoiceUpdateMaxRetries    = 5
	VoiceUpdateRetryInterval = 100 * time.Millisecond
	VoiceChannelTimeout      = 1 * time.Minute

	EmbedColor        = 0xFF4500
	EmbedSuccessColor = 0x00FF00
	EmbedErrorColor   = 0xFF0000
	EmbedMaxLength    = 1000

	TopServersAmount  = 100
	ServersShownLimit = 15
	TracksShownLimit  = 10
	BarLength         = 20
	CodeBarLength     = 26

	TempKeySuffix  = "_tmp"
	GuildsKey      = "guild:%d"
	GuildPrefixKey = "guild_prefix:%d"
	StatsKey       = "bot_stats"
	BlacklistKey   = "blacklists"
	BotAdminKey    = "bot_admins"
	BotOwnerKey    = "bot_owners"
	StatusKey      = "gateway_statuses"
	StartedKey     = "gateway_started"
	ShardKey       = "gateway_shards"
)
