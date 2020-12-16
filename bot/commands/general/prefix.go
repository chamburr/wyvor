package general

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
)

var PrefixCommand = &command.Command{
	Name: "prefix",
	Arguments: []*command.Argument{
		{Name: "new prefix", Type: command.ArgumentString},
	},
	Run: func(data *command.CommandData) (res interface{}, err error) {
		res = "The prefix for this server is `" + data.GuildPrefix + "`."

		newPrefix, _ := data.Arg(0)
		if newPrefix != nil {
			_, err = api.RequestPatch(data.Author, api.EndpointGuildSettings(data.Guild.ID), map[string]interface{}{
				"prefix": newPrefix.(string),
			})
			if err != nil {
				return
			}
			res = "The prefix for this server is changed to `" + newPrefix.(string) + "`."
		}

		return
	},
}
