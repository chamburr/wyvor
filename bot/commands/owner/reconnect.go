package owner

import (
	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/core/queue"
	"github.com/chamburr/wyvor/utils"
)

var ReconnectCommand = &command.Command{
	Name:     "reconnect",
	Disabled: true,
	Arguments: []*command.Argument{
		{Name: "shard", Type: command.ArgumentInt, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotOwner,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		shard, _ := data.Arg(0)

		if shard.(int) <= common.Shards {
			return utils.ErrorEmbed("The specified shard does not exist."), nil
		}

		err = queue.Reconnect(shard.(int))
		if err != nil {
			return nil, err
		}

		return "Reconnecting to the shard.", nil
	},
}
