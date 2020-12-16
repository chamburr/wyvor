package core

import (
	"fmt"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var EffectsCommand = &command.Command{
	Name:    "effects",
	Aliases: []string{"filters"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		player, err := api.RequestGet(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		effects := player["filters"].(map[string]interface{})

		content := "__**Timescale**__\n```"
		content += fmt.Sprintf("Speed:          %.1f\n", utils.ToFloat64(effects["timescale"].(map[string]interface{})["speed"]))
		content += fmt.Sprintf("Pitch:          %.1f\n", utils.ToFloat64(effects["timescale"].(map[string]interface{})["pitch"]))
		content += fmt.Sprintf("Rate:           %.1f\n", utils.ToFloat64(effects["timescale"].(map[string]interface{})["rate"]))
		content += "```\n__**Tremolo**__\n```"
		content += fmt.Sprintf("Depth:          %.1f\n", utils.ToFloat64(effects["tremolo"].(map[string]interface{})["depth"]))
		content += fmt.Sprintf("Frequency:      %.1f\n", utils.ToFloat64(effects["tremolo"].(map[string]interface{})["frequency"]))
		content += "```\n__**Vibrato**__\n```"
		content += fmt.Sprintf("Depth:          %.1f\n", utils.ToFloat64(effects["vibrato"].(map[string]interface{})["depth"]))
		content += fmt.Sprintf("Frequency:      %.1f\n", utils.ToFloat64(effects["vibrato"].(map[string]interface{})["frequency"]))
		content += "```\n__**Karaoke**__\n```"
		content += fmt.Sprintf("Level:          %.1f\n", utils.ToFloat64(effects["karaoke"].(map[string]interface{})["level"]))
		content += fmt.Sprintf("Mono Level:     %.1f\n", utils.ToFloat64(effects["karaoke"].(map[string]interface{})["monoLevel"]))
		content += fmt.Sprintf("Band:           %.1f\n", utils.ToFloat64(effects["karaoke"].(map[string]interface{})["filterBand"]))
		content += fmt.Sprintf("Width:          %.1f\n", utils.ToFloat64(effects["karaoke"].(map[string]interface{})["filterWidth"]))
		content += "```\n"
		content += fmt.Sprintf("[Adjust the effects here](%s/dashboard/%d/effects)", config.BaseUri.GetString(), data.Guild.ID)

		return content, nil
	},
}
