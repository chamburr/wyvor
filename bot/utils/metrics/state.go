package metrics

import (
	"time"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/cache"
)

func RunJobFlushState() {
	lastHits := int64(0)
	lastMisses := int64(0)
	lastEvictionsCache := int64(0)
	lastEvictionsMembers := int64(0)

	ticker := time.NewTicker(config.MetricsFlushInterval)
	for {
		<-ticker.C

		var botStats cache.Stats
		err := cache.GetDataJson(config.StatsKey, &botStats)
		if err != nil {
			log.WithError(err).Error("Failed to retrieve bot stats")
			continue
		}

		StateGuilds.Set(float64(botStats.Guilds))
		StateChannels.Set(float64(botStats.Channels))
		StateRoles.Set(float64(botStats.Roles))
		StateMembers.Set(float64(botStats.Members))
		StateVoices.Set(float64(botStats.Voices))

		stats := common.State.StateStats()

		StateCacheHits.Add(float64(stats.CacheHits - lastHits))
		StateCacheMisses.Add(float64(stats.CacheMisses - lastMisses))
		StateCacheEvictions.Add(float64(stats.UserCachceEvictedTotal - lastEvictionsCache))
		StateCacheMemberEvictions.Add(float64(stats.MembersRemovedTotal - lastEvictionsMembers))

		lastHits = stats.CacheHits
		lastMisses = stats.CacheMisses
		lastEvictionsCache = stats.UserCachceEvictedTotal
		lastEvictionsMembers = stats.MembersRemovedTotal
	}
}
