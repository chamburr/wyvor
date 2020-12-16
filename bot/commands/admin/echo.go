package admin

import (
	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/core/command"
)

var EchoCommand = &command.Command{
	Name: "echo",
	Arguments: []*command.Argument{
		{Name: "content", Type: command.ArgumentString, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotAdmin,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		content, _ := data.Arg(0)

		_ = common.Session.ChannelMessageDelete(data.Channel.ID, data.Message.ID)
		_, err = common.Session.ChannelMessageSend(data.Channel.ID, content.(string))

		return
	},
}
