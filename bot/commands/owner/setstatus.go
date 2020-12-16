package owner

import (
	"fmt"

	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/core/queue"
	"github.com/jonas747/discordgo"
)

var SetStatusCommand = &command.Command{
	Name: "set_status",
	Arguments: []*command.Argument{
		{Name: "status", Type: command.ArgumentString, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotOwner,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		status, _ := data.Arg(0)

		err = queue.SetStatus(discordgo.GameTypeGame, status.(string))
		if err != nil {
			return
		}

		return fmt.Sprintf("Changed the bot status."), nil
	},
}
