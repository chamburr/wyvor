package queue

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var PreviousCommand = &command.Command{
	Name:    "previous",
	Aliases: []string{"back", "prev"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		player, err := api.RequestGet(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		playing := utils.ToInt64(player["playing"])
		if playing == -1 {
			queue, err := api.RequestGetArray(data.Author, api.EndpointGuildQueue(data.Guild.ID))
			if err != nil {
				return nil, err
			}

			playing = int64(len(queue))
		}

		_, err = api.RequestPatch(data.Author, api.EndpointGuildPlayer(data.Guild.ID), map[string]interface{}{
			"playing": playing - 1,
		})
		if err != nil {
			return
		}

		return "Went back to the previous track.", nil
	},
}
