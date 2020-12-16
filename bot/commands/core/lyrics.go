package core

import (
	"fmt"
	"net/url"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var LyricsCommand = &command.Command{
	Name:    "lyrics",
	Aliases: []string{"l"},
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

		lyrics, err := api.RequestGet(data.Author, api.EndpointTrackLyrics(url.QueryEscape(playing["track"].(string))))
		if err != nil {
			return nil, err
		}

		res = "__**" + lyrics["title"].(string) + "**__\n\n" + lyrics["content"].(string)
		if len(res.(string)) > config.EmbedMaxLength {
			hint := fmt.Sprintf("\n\n[View the full lyrics here](%s/dashboard/%d/lyrics)", config.BaseUri.GetString(), data.Guild.ID)
			res = res.(string)[:config.EmbedMaxLength-len(hint)] + hint
		}

		return
	},
}
