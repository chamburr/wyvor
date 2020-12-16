package playlist

import (
	"fmt"
	"strings"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var PlaylistShowCommand = &command.Command{
	Name:    "playlist show",
	Aliases: []string{"pl show", "playlist view", "pl view"},
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

		content := fmt.Sprintf("**%s** - <@%d>\n\n", selected["name"].(string), utils.ToInt64(selected["author"]))
		for index, track := range selected["items"].([]interface{}) {
			if index >= config.TracksShownLimit {
				break
			}
			track := track.(map[string]interface{})
			content += fmt.Sprintf("%d. %s `[%s]`\n", index+1, utils.FormatTrack(track, false), utils.FormatPosition(utils.ToInt64(track["length"])))
		}

		content += fmt.Sprintf("\n[View the full playlist here](%s/dashboard/%d/playlists)", config.BaseUri.GetString(), data.Guild.ID)

		return content, nil
	},
}
