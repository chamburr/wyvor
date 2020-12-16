package queue

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var RemoveCommand = &command.Command{
	Name:    "remove",
	Aliases: []string{"rm", "delete", "del"},
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

		_, err = api.RequestDelete(data.Author, api.EndpointGuildQueueItem(data.Guild.ID, item.(int64)))
		if err != nil {
			return
		}

		return "Removed " + utils.FormatTrack(queue[item.(int64)].(map[string]interface{}), false) + " from the queue.", nil
	},
}
