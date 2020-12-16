package core

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var DisconnectCommand = &command.Command{
	Name:    "disconnect",
	Aliases: []string{"leave", "dc"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		_, err = api.RequestDelete(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			return
		}

		return "Disconnected from the channel.", nil
	},
}
