package general

import (
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/command"
)

var SupportCommand = &command.Command{
	Name:    "support",
	Aliases: []string{"server"},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		return "Join our support server [here](" + config.BaseUri.GetString() + "/support).", nil
	},
}
