package admin

import (
	"fmt"
	"strconv"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var TopServersCommand = &command.Command{
	Name: "top_servers",
	Arguments: []*command.Argument{
		{Name: "amount", Type: command.ArgumentInt},
	},
	PermChecks: []command.PermFunc{
		command.BotAdmin,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		amount := config.TopServersAmount

		argAmount, _ := data.Arg(0)
		if argAmount != nil {
			amount = int(argAmount.(int64))
		}

		guilds, err := api.RequestGetArray(data.Author, fmt.Sprintf(api.EndpointAdminTopGuilds, amount))
		if err != nil {
			return nil, err
		}

		content := ""
		for index, guild := range guilds {
			content += strconv.Itoa(index+1) + ". " + utils.FormatGuild(guild.(map[string]interface{})) + "\n"
		}

		res, err = utils.ShortenContent(content, config.ServersShownLimit)
		if err != nil {
			return
		}

		return
	},
}
