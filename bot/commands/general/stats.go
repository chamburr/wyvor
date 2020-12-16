package general

import (
	"fmt"
	"runtime"
	"strconv"
	"time"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/cache"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
	"github.com/jonas747/discordgo"
	"github.com/shirou/gopsutil/cpu"
	"github.com/shirou/gopsutil/host"
	"github.com/shirou/gopsutil/mem"
)

var StatsCommand = &command.Command{
	Name:    "stats",
	Aliases: []string{"statistics"},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		var memRuntime runtime.MemStats
		runtime.ReadMemStats(&memRuntime)

		memStats, err := mem.VirtualMemory()
		if err != nil {
			return nil, err
		}

		cpuPercent, err := cpu.Percent(0, false)
		if err != nil {
			return nil, err
		}

		hostInfo, err := host.Info()
		if err != nil {
			return nil, err
		}

		started, err := cache.GetData(config.StartedKey)
		if err != nil {
			return nil, err
		}

		timestamp, err := time.Parse(time.RFC3339, started+"Z")
		if err != nil {
			return nil, err
		}

		players, err := api.RequestGet(data.Author, api.EndpointStatsPlayer)
		if err != nil {
			return nil, err
		}

		users := 0
		for _, guild := range common.State.GuildsSlice(true) {
			users += guild.Guild.MemberCount
		}

		shard := 0
		if data.Guild != nil {
			shard = utils.ShardForGuild(data.Guild.ID)
		}

		embed := utils.DefaultEmbed(data.Guild)
		embed.Title = common.BotUser.Username + " Statistics"
		embed.Description = "Visit the bot status page [here](" + config.BaseUri.GetString() + "/status) for more information."
		embed.Thumbnail = &discordgo.MessageEmbedThumbnail{URL: common.BotUser.AvatarURL("")}
		embed.Fields = []*discordgo.MessageEmbedField{
			{Name: "Owner", Value: "CHamburr#2591", Inline: true},
			{Name: "Version", Value: common.Version, Inline: true},
			{Name: "Uptime", Value: utils.FormatDuration(time.Since(timestamp)), Inline: true},
			{Name: "Servers", Value: strconv.Itoa(len(common.State.Guilds)), Inline: true},
			{Name: "Users", Value: strconv.Itoa(users), Inline: true},
			{Name: "Players", Value: strconv.Itoa(int(utils.ToInt64(players["players"]))), Inline: true},
			{Name: "Platform", Value: fmt.Sprintf("%s %s", hostInfo.Platform, hostInfo.PlatformVersion), Inline: true},
			{Name: "CPU Usage", Value: fmt.Sprintf("%.1f%%", cpuPercent[0]), Inline: true},
			{Name: "RAM Usage", Value: fmt.Sprintf("%.1f%%", memStats.UsedPercent), Inline: true},
		}
		embed.Footer = &discordgo.MessageEmbedFooter{
			Text: fmt.Sprintf("Shard %d/%d", shard, common.Shards),
		}

		return embed, nil
	},
}
