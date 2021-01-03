package core

import (
	"fmt"
	"net/http"
	"net/url"

	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var PlayCommand = &command.Command{
	Name:    "play",
	Aliases: []string{"p"},
	Arguments: []*command.Argument{
		{Name: "query", Type: command.ArgumentString, Required: true},
	},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		query, _ := data.Arg(0)

		tracks, err := api.RequestGetArray(data.Author, fmt.Sprintf(api.EndpointTrackQuery, url.QueryEscape(query.(string))))
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
