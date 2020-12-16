package player

import (
	"time"

	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var RewindCommand = &command.Command{
	Name:    "rewind",
	Aliases: []string{"rw", "rwd"},
	Arguments: []*command.Argument{
		{Name: "amount", Type: command.ArgumentDuration, Required: true},
	},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		amount, _ := data.Arg(0)

		player, err := api.RequestGet(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		newPosition := utils.ToInt64(player["position"]) - amount.(time.Duration).Milliseconds()

		_, err = api.RequestPatch(data.Author, api.EndpointGuildPlayer(data.Guild.ID), map[string]interface{}{
			"position": newPosition,
		})
		if err != nil {
			return
		}

		return "The player is rewound to **" + utils.FormatPosition(newPosition) + "**.", nil
	},
}
