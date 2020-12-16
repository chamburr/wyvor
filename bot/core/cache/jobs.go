package cache

import (
	"fmt"
	"time"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/jonas747/discordgo"
	"github.com/mediocregopher/radix/v3"
)

func RunJobFlushGuilds() {
	ticker := time.NewTicker(config.RedisGuildsFlushInterval)
	for {
		<-ticker.C

		for _, guild := range common.State.GuildsSlice(true) {
			guild := Guild{
				ID:          guild.ID,
				Name:        guild.Guild.Name,
				Icon:        guild.Guild.Icon,
				Owner:       guild.Guild.OwnerID,
				MemberCount: guild.Guild.MemberCount,
			}

			key := fmt.Sprintf(config.GuildsKey, guild.ID)
			err := SetDataJsonExpire(key, guild, config.RedisGuildsCacheExpiry)
			if err != nil {
				log.WithError(err).Error("Failed to flush guilds")
				break
			}
		}

	}
}

func RunJobFlushStats() {
	ticker := time.NewTicker(config.RedisStatsFlushInterval)
	for {
		<-ticker.C

		members := 0
		roles := 0
		voices := 0
		for _, guild := range common.State.GuildsSlice(true) {
			members += len(guild.Members)
			roles += len(guild.Guild.Roles)

			if guild.VoiceState(true, common.BotUser.ID) != nil {
				voices += 1
			}
		}

		started, err := GetData(config.StartedKey)
		if err != nil {
			log.WithError(err).Error("Failed to get started time")
			continue
		}

		stats := Stats{
			Version:  common.Version,
			Started:  started,
			Shards:   common.Shards,
			Guilds:   len(common.State.Guilds),
			Roles:    roles,
			Channels: len(common.State.Channels),
			Members:  members,
			Voices:   voices,
		}

		err = SetDataJson(config.StatsKey, stats)
		if err != nil {
			log.WithError(err).Error("Failed to flush stats")
		}
	}
}

func RunJobFlushAdmins() {
	ticker := time.NewTicker(config.RedisAdminsFlushInterval)
	for {
		<-ticker.C

		if guild := common.State.Guild(true, config.MainGuild.GetInt64()); guild != nil {
			guild.RLock()
			for _, member := range guild.Members {
				isAdmin := false
				isOwner := false
				for _, role := range member.Roles {
					if role == config.BotAdminRole.GetInt64() {
						isAdmin = true
					}

					if role == config.BotOwnerRole.GetInt64() {
						isAdmin = true
						isOwner = true
						break
					}
				}

				if isAdmin == true {
					err := common.Redis.Do(radix.Cmd(nil, "SADD", config.BotAdminKey+config.TempKeySuffix, discordgo.StrID(member.ID)))
					if err != nil {
						log.WithError(err).Error("Failed to update admins")
						break
					}
				}

				if isOwner == true {
					err := common.Redis.Do(radix.Cmd(nil, "SADD", config.BotOwnerKey+config.TempKeySuffix, discordgo.StrID(member.ID)))
					if err != nil {
						log.WithError(err).Error("Failed to update owners")
						break
					}
				}
			}
			guild.RUnlock()

			err := common.Redis.Do(radix.Cmd(nil, "RENAME", config.BotAdminKey+config.TempKeySuffix, config.BotAdminKey))
			if err != nil {
				log.WithError(err).Error("Failed to flush admins")
			}

			err = common.Redis.Do(radix.Cmd(nil, "RENAME", config.BotOwnerKey+config.TempKeySuffix, config.BotOwnerKey))
			if err != nil {
				log.WithError(err).Error("Failed to flush owners")
			}
		}
	}
}
