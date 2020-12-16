package queue

import (
	"fmt"

	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var ClearCommand = &command.Command{
	Name: "clear",
	Run: func(data *command.CommandData) (res interface{}, err error) {
		queue, err := api.RequestGetArray(data.Author, api.EndpointGuildQueue(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		_, err = api.RequestDelete(data.Author, api.EndpointGuildQueue(data.Guild.ID))
		if err != nil {
			return
		}

		return fmt.Sprintf("Removed %d tracks from the queue.", len(queue)), nil
	},
}
