package utils

import (
	"encoding/json"
	"fmt"
	"math"
	"sort"
	"strings"
	"time"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/jonas747/discordgo"
	"github.com/jonas747/dstate/v2"
)

var (
	log = logger.WithPrefix("utils")
)

func ShardForGuild(guild int64) int {
	return int(guild >> 22 % int64(common.Shards))
}

func FormatTime(value time.Time) string {
	timestamp := value.Format(time.RFC3339Nano)
	if strings.Contains(timestamp, "Z") {
		return strings.Split(timestamp, "Z")[0]
	} else {
		return strings.Split(timestamp, "+")[0]
	}
}

func FormatDuration(value time.Duration) string {
	duration := ""

	hours := int(math.Floor(value.Hours()))
	minutes := int(math.Floor(value.Minutes()))
	seconds := int(math.Floor(value.Seconds()))

	days := hours / 24
	hours = hours % 24
	minutes = minutes % 60
	seconds = seconds % 60

	if days > 0 {
		duration += fmt.Sprintf("%dd ", days)
	}

	duration += fmt.Sprintf("%dh %dm %ds", hours, minutes, seconds)

	return duration
}

func FormatPosition(value int64) string {
	duration := time.Duration(value) * time.Millisecond

	if duration.Hours() >= 1 {
		return fmt.Sprintf("%02d:%02d:%02d", int(duration.Hours()), int(duration.Minutes())%60, int(duration.Seconds())%60)
	} else {
		return fmt.Sprintf("%02d:%02d", int(duration.Minutes()), int(duration.Seconds())%60)
	}
}

func FormatTrack(track map[string]interface{}, user bool) string {
	result := fmt.Sprintf("[%s](%s)", track["title"].(string), track["uri"].(string))
	if user == true {
		result += fmt.Sprintf(" - <@%d>", ToInt64(track["author"]))
	}

	return result
}

func FormatGuild(guild map[string]interface{}) string {
	return fmt.Sprintf("%s `%d` (%d members)", guild["name"], ToInt64(guild["id"]), ToInt64(guild["member_count"]))
}

func FormatBar(amount float64, minimum float64, maximum float64, code bool) string {
	before := (amount + minimum) / (maximum - minimum)
	after := 0.0

	var dash string
	var circle string

	if code == true {
		dash = "—"
		circle = "◯"
		before = math.Abs(before * config.CodeBarLength)
		after = config.CodeBarLength - before
	} else {
		dash = "▬"
		circle = "⚪"
		before = math.Abs(before * config.BarLength)
		after = config.BarLength - before
	}

	return strings.Repeat(dash, int(before)) + circle + strings.Repeat(dash, int(after))
}

func ToInt64(value interface{}) int64 {
	result, _ := value.(json.Number).Int64()
	return result
}

func ToFloat64(value interface{}) float64 {
	result, _ := value.(json.Number).Float64()
	return result
}

func ShortenCode(content string) (string, error) {
	code := "```\n" + content + "```"

	if len(code) <= config.EmbedMaxLength {
		return code, nil
	}

	url, err := Upload(content)
	if err != nil {
		return "", err
	}

	output := "```\n [View the full output here](" + url + ")"
	output = code[:config.EmbedMaxLength-len(output)] + output

	return output, nil
}

func ShortenContent(content string, maxLines int) (string, error) {
	lines := strings.Split(content, "\n")

	output := ""
	for index, line := range lines {
		if maxLines == -1 || index < maxLines {
			output += line + "\n"
		}
	}

	if len(lines) > maxLines {
		url, err := Upload(content)
		if err != nil {
			return "", err
		}

		output += "\n [View the full output here](" + url + ")"
	}

	return output, nil
}

func BotHasPermission(guild *dstate.GuildState, channel *dstate.ChannelState, permissions ...int) bool {
	if guild == nil {
		return true
	}

	perms, err := guild.MemberPermissions(true, channel.ID, common.BotUser.ID)
	if err != nil && err != dstate.ErrChannelNotFound {
		log.WithError(err).WithField("guild", guild.ID).Error("Failed to check permissions")
		return true
	}

	if perms&discordgo.PermissionAdministrator != 0 {
		return true
	}

	for _, permission := range permissions {
		if perms&permission != permission {
			return false
		}
	}

	return false
}

func WaitConnect(guild *dstate.GuildState) *discordgo.VoiceState {
	retries := 0
	ticker := time.NewTicker(config.VoiceUpdateRetryInterval)

	for {
		<-ticker.C

		if state := guild.VoiceState(true, common.BotUser.ID); state != nil {
			return state
		}

		retries += 1
		if retries >= config.VoiceUpdateMaxRetries {
			log.WithField("guild", guild.ID).Warn("Timed out waiting for voice state")
			break
		}
	}

	return nil
}

func VoiceConnections(guild *dstate.GuildState, channel int64) int {
	voices := 0

	guild.RLock()
	for _, userVoice := range guild.Guild.VoiceStates {
		if userVoice.ChannelID == channel {
			voices += 1
		}
	}
	guild.RUnlock()

	return voices
}

func GuildColor(guild *dstate.GuildState) int {
	if guild == nil {
		return config.EmbedColor
	}

	if member := guild.Member(true, common.BotUser.ID); member != nil {
		guildRoles := discordgo.Roles(guild.Guild.Roles)
		sort.Sort(guildRoles)

		for _, guildRole := range guildRoles {
			if guildRole.Color == 0 {
				continue
			}

			for _, memberRole := range member.Roles {
				if guildRole.ID == memberRole {
					return guildRole.Color
				}
			}
		}
	}

	return config.EmbedColor
}

func DefaultEmbed(guild *dstate.GuildState) *discordgo.MessageEmbed {
	embed := &discordgo.MessageEmbed{
		Color: GuildColor(guild),
	}

	return embed
}

func ErrorEmbed(message string) *discordgo.MessageEmbed {
	embed := &discordgo.MessageEmbed{
		Title:       "An Error Occurred",
		Description: message,
		Color:       config.EmbedErrorColor,
	}

	return embed
}
