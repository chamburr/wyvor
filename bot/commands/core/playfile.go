package core

import (
	"fmt"
	"net/http"
	"net/url"

	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var PlayFileCommand = &command.Command{
	Name:    "play_file",
	Aliases: []string{"pf"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		if len(data.Message.Attachments) == 0 {
			return utils.ErrorEmbed("There are no files attached to the command."), nil
		}

		tracks, err := api.RequestGetArray(data.Author, fmt.Sprintf(api.EndpointTrackQuery, url.QueryEscape(data.Message.Attachments[0].URL)))
		if err != nil {
			return nil, err
		}

		if len(tracks) == 0 {
			return utils.ErrorEmbed("There are no tracks found."), nil
		}

		_, err = api.RequestGet(data.Author, api.EndpointGuildPlayer(data.Guild.ID))
		if err != nil {
			switch err.(type) {
			case api.ApiError:
				if err.(api.ApiError).Response.StatusCode == http.StatusBadRequest {
					_, err = api.RequestPost(data.Author, api.EndpointGuildPlayer(data.Guild.ID), nil)
					if err != nil {
						return
					}
					if utils.WaitConnect(data.Guild) == nil {
						return
					}
				} else {
					return
				}
			default:
				return
			}
		}

		track := tracks[0].(map[string]interface{})

		_, err = api.RequestPost(data.Author, api.EndpointGuildQueue(data.Guild.ID), map[string]interface{}{
			"track": track["track"].(string),
		})
		if err != nil {
			return nil, err
		}

		return "Added " + utils.FormatTrack(track, false) + " to the queue.", nil
	},
}
