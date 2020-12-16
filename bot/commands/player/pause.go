package player

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var PauseCommand = &command.Command{
	Name:    "pause",
	Aliases: []string{"stop"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		_, err = api.RequestPatch(data.Author, api.EndpointGuildPlayer(data.Guild.ID), map[string]interface{}{
			"paused": true,
		})
		if err != nil {
			return
		}

		return "The player is paused.", nil
	},
}
