package owner

import (
	"fmt"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/core/queue"
)

var DisconnectAllCommand = &command.Command{
	Name:    "disconnect_all",
	Aliases: []string{"dcall"},
	PermChecks: []command.PermFunc{
		command.BotOwner,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		voices := 0
		for _, guild := range common.State.GuildsSlice(true) {
			if voice := guild.VoiceState(true, common.BotUser.ID); voice != nil {
				voices += 1
				err = queue.SetVoiceState(guild.ID, 0)
				if err != nil {
					return
				}
			}
		}

		return fmt.Sprintf("Disconnected from %d voice channels.", voices), nil
	},
}
