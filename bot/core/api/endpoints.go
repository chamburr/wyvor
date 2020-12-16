package api

import (
	"github.com/jonas747/discordgo"
)

var (
	EndpointAdmin  = "/admin"
	EndpointGuilds = "/guilds"
	EndpointTracks = "/tracks"

	EndpointStats       = "/stats"
	EndpointStatsPlayer = EndpointStats + "/player"

	EndpointAdminGuildsName    = EndpointAdmin + "/guilds?name=%s"
	EndpointAdminGuildsOwner   = EndpointAdmin + "/guilds?&owner=%d"
	EndpointAdminTopGuilds     = EndpointAdmin + "/top_guilds?amount=%d"
	EndpointAdminBlacklist     = EndpointAdmin + "/blacklist"
	EndpointAdminBlacklistItem = func(id int64) string { return EndpointAdminBlacklist + "/" + discordgo.StrID(id) }

	EndpointGuild                  = func(id int64) string { return EndpointGuilds + "/" + discordgo.StrID(id) }
	EndpointGuildSettings          = func(id int64) string { return EndpointGuild(id) + "/settings" }
	EndpointGuildPlayer            = func(id int64) string { return EndpointGuild(id) + "/player" }
	EndpointGuildQueue             = func(id int64) string { return EndpointGuild(id) + "/queue" }
	EndpointGuildQueueShuffle      = func(id int64) string { return EndpointGuild(id) + "/queue/shuffle" }
	EndpointGuildQueueItem         = func(id int64, item int64) string { return EndpointGuild(id) + "/queue/" + discordgo.StrID(item) }
	EndpointGuildQueueItemPosition = func(id int64, item int64) string { return EndpointGuildQueueItem(id, item) + "/position" }
	EndpointGuildPlaylists         = func(id int64) string { return EndpointGuild(id) + "/playlists" }
	EndpointGuildPlaylist          = func(id int64, item int64) string { return EndpointGuildPlaylists(id) + "/" + discordgo.StrID(item) }
	EndpointGuildPlaylistLoad      = func(id int64, item int64) string { return EndpointGuildPlaylist(id, item) + "/load" }

	EndpointTrackQuery  = EndpointTracks + "?query=%s"
	EndpointTrackLyrics = func(id string) string { return EndpointTracks + "/lyrics?id=" + id }
)
