package queue

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var NextCommand = &command.Command{
	Name:    "next",
	Aliases: []string{"skip", "s"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		player, err := api.RequestGet(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		queue, err := api.RequestGetArray(data.Author, api.EndpointGuildQueue(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		playing := utils.ToInt64(player["playing"])
		if playing + 1 == int64(len(queue)) {
			playing = -2
		}

		_, err = api.RequestPatch(data.Author, api.EndpointGuildPlayer(data.Guild.ID), map[string]interface{}{
			"playing": playing + 1,
		})
		if err != nil {
			return
		}

		return "Skipped to the next track.", nil
	},
}
