package playlist

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var PlaylistCreateCommand = &command.Command{
	Name:    "playlist create",
	Aliases: []string{"pl create", "playlist new", "pl new"},
	Arguments: []*command.Argument{
		{Name: "name", Type: command.ArgumentString, Required: true},
	},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		name, _ := data.Arg(0)

		_, err = api.RequestPost(data.Author, api.EndpointGuildPlaylists(data.Guild.ID), map[string]interface{}{
			"name": name.(string),
		})
		if err != nil {
			return
		}

		return "Created a new playlist '" + name.(string) + "'.", nil
	},
}
