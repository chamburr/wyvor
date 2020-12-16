package core

import (
	"strings"

	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
	"github.com/jonas747/discordgo"
)

var NowPlayingCommand = &command.Command{
	Name:    "now_playing",
	Aliases: []string{"np", "playing"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		queue, err := api.RequestGetArray(data.Author, api.EndpointGuildQueue(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		player, err := api.RequestGet(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		if utils.ToInt64(player["playing"]) == -1 {
			return "Nothing is currently playing.", nil
		}

		playing := queue[utils.ToInt64(player["playing"])].(map[string]interface{})

		embed := utils.DefaultEmbed(data.Guild)
		embed.Title = "Now Playing"
		embed.Description = utils.FormatTrack(playing, true)
		embed.Footer = &discordgo.MessageEmbedFooter{Text: ""}
		embed.Footer.Text += utils.FormatBar(utils.ToFloat64(player["position"]), 0, utils.ToFloat64(playing["length"]), false) + " "
		embed.Footer.Text += utils.FormatPosition(utils.ToInt64(player["position"])) + " / " + utils.FormatPosition(utils.ToInt64(playing["length"]))

		if strings.Contains(playing["uri"].(string), "youtube.com/") {
			embed.Thumbnail = &discordgo.MessageEmbedThumbnail{
				URL: "https://img.youtube.com/vi/" + strings.Split(playing["uri"].(string), "?v=")[1] + "/mqdefault.jpg",
			}
		}

		return embed, nil
	},
}
