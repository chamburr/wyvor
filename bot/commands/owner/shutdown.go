package owner

import (
	"github.com/chamburr/wyvor/core"
	"github.com/chamburr/wyvor/core/command"
)

var ShutdownCommand = &command.Command{
	Name: "shutdown",
	PermChecks: []command.PermFunc{
		command.BotOwner,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		defer core.Shutdown()

		return "Shutting down the bot.", nil
	},
}
