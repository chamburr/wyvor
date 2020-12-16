package playlist

import (
	"strings"

	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var PlaylistLoadCommand = &command.Command{
	Name:    "playlist load",
	Aliases: []string{"pl load"},
	Arguments: []*command.Argument{
		{Name: "name", Type: command.ArgumentString, Required: true},
	},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		name, _ := data.Arg(0)

		playlists, err := api.RequestGetArray(data.Author, api.EndpointGuildPlaylists(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		var selected map[string]interface{}
		for _, playlist := range playlists {
			if strings.ToLower(playlist.(map[string]interface{})["name"].(string)) == strings.ToLower(name.(string)) {
				selected = playlist.(map[string]interface{})
				break
			}
		}

		if selected == nil {
			return utils.ErrorEmbed("The specified playlist could not be found."), nil
		}

		_, err = api.RequestPost(data.Author, api.EndpointGuildPlaylistLoad(data.Guild.ID, utils.ToInt64(selected["id"])), nil)
		if err != nil {
			return
		}

		return "Loaded the playlist '" + selected["name"].(string) + "' to queue.", nil
	},
}
