package core

import (
	"fmt"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var EqualizerCommand = &command.Command{
	Name:    "equalizer",
	Aliases: []string{"eq"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		player, err := api.RequestGet(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		content := "```\n"
		for _, equalizer := range player["filters"].(map[string]interface{})["equalizer"].(map[string]interface{})["bands"].([]interface{}) {
			gain := utils.ToFloat64(equalizer.(map[string]interface{})["gain"])
			if gain < 0 {
				gain = gain * 4
			}

			content += fmt.Sprintf("B%-2d %s\n", utils.ToInt64(equalizer.(map[string]interface{})["band"]), utils.FormatBar(gain, -1, 1, true))
		}
		content += "```\n"
		content += fmt.Sprintf("[Adjust the equalizer here](%s/dashboard/%d/equalizer)", config.BaseUri.GetString(), data.Guild.ID)

		return content, nil
	},
}
