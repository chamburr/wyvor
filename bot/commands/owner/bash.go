package owner

import (
	"os/exec"

	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
)

var BashCommand = &command.Command{
	Name: "bash",
	Arguments: []*command.Argument{
		{Name: "command", Type: command.ArgumentString, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotOwner,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		content, _ := data.Arg(0)

		stdout, err := exec.Command("bash", "-c", content.(string)).Output()
		if err != nil {
			return utils.ErrorEmbed(err.Error()), nil
		}

		res, err = utils.ShortenCode(string(stdout))
		if err != nil {
			return
		}

		return
	},
}
