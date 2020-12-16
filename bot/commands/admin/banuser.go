package admin

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/jonas747/discordgo"
)

var BanUserCommand = &command.Command{
	Name: "ban_user",
	Arguments: []*command.Argument{
		{Name: "user", Type: command.ArgumentUser, Required: true},
		{Name: "reason", Type: command.ArgumentString, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotAdmin,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		user, _ := data.Arg(0)
		reason, _ := data.Arg(1)

		_, err = api.RequestPut(data.Author, api.EndpointAdminBlacklistItem(user.(*discordgo.User).ID), map[string]interface{}{
			"reason": reason.(string),
		})
		if err != nil {
			return
		}

		return "The user is banned from the bot.", nil
	},
}
