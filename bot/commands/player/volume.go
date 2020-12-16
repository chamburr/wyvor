package player

import (
	"fmt"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var VolumeCommand = &command.Command{
	Name:    "volume",
	Aliases: []string{"vol"},
	Arguments: []*command.Argument{
		{Name: "loudness", Type: command.ArgumentInt, Required: true},
	},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		loudness, _ := data.Arg(0)

		_, err = api.RequestPatch(data.Author, api.EndpointGuildPlayer(data.Guild.ID), map[string]interface{}{
			"volume": loudness.(int64),
		})
		if err != nil {
			return
		}

		return fmt.Sprintf("The player volume is changed to **%d%%**.", loudness.(int64)), nil
	},
}
