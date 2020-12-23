package general

import (
	"net/url"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/command"
)

var CommandsCommand = &command.Command{
	Name: "commands",
	Aliases: []string{"c"},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		return "View the full list of commands [here](" + config.BaseUri.GetString() + "/commands?prefix=" + url.QueryEscape(data.GuildPrefix) + ").", nil
	},
}
