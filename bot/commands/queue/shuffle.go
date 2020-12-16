package queue

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var ShuffleCommand = &command.Command{
	Name:    "shuffle",
	Aliases: []string{"shuf"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		_, err = api.RequestPost(data.Author, api.EndpointGuildQueueShuffle(data.Guild.ID), nil)
		if err != nil {
			return
		}

		return "The queue has been shuffled.", nil
	},
}
