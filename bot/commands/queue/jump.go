package queue

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var JumpCommand = &command.Command{
	Name:    "jump",
	Aliases: []string{"goto", "j"},
	Arguments: []*command.Argument{
		{Name: "item", Type: command.ArgumentInt, Required: true},
	},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		item, _ := data.Arg(0)

		item = item.(int64) - 1

		queue, err := api.RequestGetArray(data.Author, api.EndpointGuildQueue(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		_, err = api.RequestPatch(data.Author, api.EndpointGuildPlayer(data.Guild.ID), map[string]interface{}{
			"playing": item.(int64),
		})
		if err != nil {
			return
		}

		return "Jumped to " + utils.FormatTrack(queue[item.(int64)].(map[string]interface{}), false) + ".", nil
	},
}
