package queue

import (
	"fmt"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var MoveCommand = &command.Command{
	Name:    "move",
	Aliases: []string{"m"},
	Arguments: []*command.Argument{
		{Name: "item", Type: command.ArgumentInt, Required: true},
		{Name: "position", Type: command.ArgumentInt, Required: true},
	},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		item, _ := data.Arg(0)
		position, _ := data.Arg(1)

		item = item.(int64) - 1
		position = position.(int64) - 1

		queue, err := api.RequestGetArray(data.Author, api.EndpointGuildQueue(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		_, err = api.RequestPut(data.Author, api.EndpointGuildQueueItemPosition(data.Guild.ID, item.(int64)), map[string]interface{}{
			"position": position.(int64),
		})
		if err != nil {
			return
		}

		return fmt.Sprintf("Moved %s to position %d.", utils.FormatTrack(queue[item.(int64)].(map[string]interface{}), false), position.(int64)), nil
	},
}
