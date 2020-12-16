package player

import (
	"time"

	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var SeekCommand = &command.Command{
	Name: "seek",
	Arguments: []*command.Argument{
		{Name: "position", Type: command.ArgumentDuration, Required: true},
	},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		amount, _ := data.Arg(0)

		_, err = api.RequestPatch(data.Author, api.EndpointGuildPlayer(data.Guild.ID), map[string]interface{}{
			"position": amount.(time.Duration).Milliseconds(),
		})
		if err != nil {
			return
		}

		return "The player position is changed to **" + utils.FormatPosition(amount.(time.Duration).Milliseconds()) + "**.", nil
	},
}
