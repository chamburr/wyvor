package general

import (
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/command"
)

var InviteCommand = &command.Command{
	Name:    "invite",
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		return "Invite the bot [here](" + config.BaseUri.GetString() + "/invite).", nil
	},
}
