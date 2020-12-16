package general

import (
	"net/url"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
	"github.com/jonas747/discordgo"
)

var HelpCommand = &command.Command{
	Name:    "help",
	Aliases: []string{"about", "info", "commands"},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		embed := utils.DefaultEmbed(data.Guild)
		embed.Title = common.BotUser.Username + " Help"
		embed.Description = common.BotUser.Username + " is the best feature-rich Discord music bot. Take control over your music with an intuitive dashboard, custom effects and more!\n"
		if data.Guild != nil {
			embed.Description += "\nThe prefix for this server is `" + data.GuildPrefix + "`.\n"
		}
		embed.Description += "\u200b"
		embed.Thumbnail = &discordgo.MessageEmbedThumbnail{URL: common.BotUser.AvatarURL("")}
		embed.Fields = []*discordgo.MessageEmbedField{
			{Name: "Commands", Value: "View the full list of commands [here](" + config.BaseUri.GetString() + "/commands?prefix=" + url.QueryEscape(data.GuildPrefix) + ").\n\u200b"},
			{Name: "Dashboard", Value: "Check out the dashboard for the best experience [here](" + config.BaseUri.GetString() + "/dashboard).\n\u200b"},
			{Name: "Invite", Value: "Add the bot to another server [here](" + config.BaseUri.GetString() + "/invite).\n\u200b"},
			{Name: "Support", Value: "Need further help? Join our support server [here](" + config.BaseUri.GetString() + "/support)."},
		}

		return embed, nil
	},
}
