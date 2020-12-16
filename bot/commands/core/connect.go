package core

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var ConnectCommand = &command.Command{
	Name:    "connect",
	Aliases: []string{"join"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		_, err = api.RequestPost(data.Author, api.EndpointGuildPlayer(data.Guild.ID), nil)
		if err != nil {
			return
		}

		return "Connected to the channel.", nil
	},
}
