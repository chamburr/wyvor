package player

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var ResumeCommand = &command.Command{
	Name:    "resume",
	Aliases: []string{"unpause", "continue"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		_, err = api.RequestPatch(data.Author, api.EndpointGuildPlayer(data.Guild.ID), map[string]interface{}{
			"paused": false,
		})
		if err != nil {
			return
		}

		return "The player is resumed.", nil
	},
}
