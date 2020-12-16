package player

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var LoopCommand = &command.Command{
	Name: "loop",
	Run: func(data *command.CommandData) (res interface{}, err error) {
		looping := "none"

		player, err := api.RequestGet(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		if player["looping"].(string) == "none" {
			looping = "queue"
		} else if player["looping"].(string) == "queue" {
			looping = "track"
		}

		_, err = api.RequestPatch(data.Author, api.EndpointGuildPlayer(data.Guild.ID), map[string]interface{}{
			"looping": looping,
		})
		if err != nil {
			return
		}

		return "The player loop is changed to **" + looping + "**.", nil
	},
}
