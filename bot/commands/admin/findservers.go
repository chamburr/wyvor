package admin

import (
	"fmt"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var FindServersCommand = &command.Command{
	Name: "find_servers",
	Arguments: []*command.Argument{
		{Name: "name", Type: command.ArgumentString, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotAdmin,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		name, _ := data.Arg(0)

		guilds, err := api.RequestGetArray(data.Author, fmt.Sprintf(api.EndpointAdminGuildsName, name))
		if err != nil {
			return nil, err
		}

		content := ""
		for _, guild := range guilds {
			content += utils.FormatGuild(guild.(map[string]interface{})) + "\n"
		}

		if content == "" {
			res = utils.ErrorEmbed("There are no servers found.")
			return
		}

		res, err = utils.ShortenContent(content, config.ServersShownLimit)
		if err != nil {
			return
		}

		return
	},
}
