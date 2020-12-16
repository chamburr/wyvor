package core

import (
	"fmt"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var QueueCommand = &command.Command{
	Name:    "queue",
	Aliases: []string{"q"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		queue, err := api.RequestGetArray(data.Author, api.EndpointGuildQueue(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		player, err := api.RequestGet(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		playing := utils.ToInt64(player["playing"])

		if len(queue) == 0 {
			return "There is nothing in the queue.", nil
		}

		var tracks []map[string]interface{}
		if playing == -1 {
			tracks = append(tracks, nil)
			for index, track := range queue {
				if index < config.TracksShownLimit {
					track.(map[string]interface{})["index"] = index + 1
					tracks = append(tracks, track.(map[string]interface{}))
				}
			}
		} else {
			for index, track := range queue {
				if int64(index)-playing >= 0 && int64(index)-playing < config.TracksShownLimit {
					track.(map[string]interface{})["index"] = index + 1
					tracks = append(tracks, track.(map[string]interface{}))
				}
			}
		}

		content := ""
		for index, track := range tracks {
			if index == 0 {
				content += "__**Now Playing**__\n"
			} else if index == 1 {
				content += "\n__**Next Up**__\n"
			}

			if track == nil {
				content += "There is nothing being currently played.\n"
				continue
			}

			length := utils.FormatPosition(utils.ToInt64(track["length"]))

			if index == 0 {
				content += fmt.Sprintf("%s `[%s]`\n", utils.FormatTrack(track, true), length)
			} else {
				content += fmt.Sprintf("%d. %s `[%s]`\n", track["index"], utils.FormatTrack(track, false), length)
			}
		}

		content += fmt.Sprintf("\n[View the full queue here](%s/dashboard/%d/player)", config.BaseUri.GetString(), data.Guild.ID)

		return content, nil
	},
}
