package admin

import (
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/jonas747/discordgo"
)

var UnbanUserCommand = &command.Command{
	Name: "unban_user",
	Arguments: []*command.Argument{
		{Name: "user", Type: command.ArgumentUser, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotAdmin,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		user, _ := data.Arg(0)

		_, err = api.RequestDelete(data.Author, api.EndpointAdminBlacklistItem(user.(*discordgo.User).ID))
		if err != nil {
			return
		}

		return "The user is unbanned from the bot.", nil
	},
}
