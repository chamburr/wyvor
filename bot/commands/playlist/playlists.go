package playlist

import (
	"fmt"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var PlaylistsCommand = &command.Command{
	Name:    "playlists",
	Aliases: []string{"playlist", "pl"},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		playlists, err := api.RequestGetArray(data.Author, api.EndpointGuildPlaylists(data.Guild.ID))
		if err != nil {
			return nil, err
		}

		content := ""
		for _, playlist := range playlists {
			playlist := playlist.(map[string]interface{})
			content += fmt.Sprintf("%s `[%d tracks]`\n", playlist["name"], len(playlist["items"].([]interface{})))
		}

		if content == "" {
			return "There are no playlists. Create one!", nil
		}

		content += fmt.Sprintf("\n[View all the playlists here](%s/dashboard/%d/playlists)", config.BaseUri.GetString(), data.Guild.ID)

		return content, nil
	},
}
