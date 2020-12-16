package owner

import (
	"github.com/chamburr/wyvor/core/command"
	"github.com/jonas747/discordgo"
)

var InvokeCommand = &command.Command{
	Name: "invoke",
	Arguments: []*command.Argument{
		{Name: "user", Type: command.ArgumentUser, Required: true},
		{Name: "command", Type: command.ArgumentString, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotOwner,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		user, _ := data.Arg(0)
		content, _ := data.Arg(1)

		message := data.Message
		message.Author = user.(*discordgo.User)
		message.Content = data.GuildPrefix + content.(string)

		_ = command.HandleCommand(data.Guild, data.Channel, message)

		return
	},
}
