package general

import (
	"fmt"
	"time"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/cache"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var PingCommand = &command.Command{
	Name:    "ping",
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		var statuses []*cache.Status
		err = cache.GetDataJson(config.StatusKey, &statuses)
		if err != nil {
			return
		}

		latency := 0
		for _, status := range statuses {
			if (data.Guild != nil && status.Shard == utils.ShardForGuild(data.Guild.ID)) || (data.Guild == nil && status.Shard == 0) {
				latency = status.Latency
				break
			}
		}

		embed := utils.DefaultEmbed(data.Guild)
		embed.Description = "Checking latency..."

		started := time.Now()

		message, err := common.Session.ChannelMessageSendEmbed(data.Channel.ID, embed)
		if err != nil {
			return nil, err
		}

		httpLatency := time.Since(started).Milliseconds()

		embed.Title = "Pong!"
		embed.Description = fmt.Sprintf("Gateway latency: %dms.\nHTTP latency: %dms.", latency, httpLatency)

		_, err = common.Session.ChannelMessageEditEmbed(message.ChannelID, message.ID, embed)

		return
	},
}
