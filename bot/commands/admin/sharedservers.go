package admin

import (
	"fmt"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
	"github.com/jonas747/discordgo"
)

var SharedServersCommand = &command.Command{
	Name: "shared_servers",
	Arguments: []*command.Argument{
		{Name: "user", Type: command.ArgumentUser, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotAdmin,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		user, _ := data.Arg(0)

		guilds, err := api.RequestGetArray(data.Author, fmt.Sprintf(api.EndpointAdminGuildsOwner, user.(*discordgo.User).ID))
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
