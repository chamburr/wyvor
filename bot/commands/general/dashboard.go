package general

import (
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/command"
)

var DashboardCommand = &command.Command{
	Name:    "dashboard",
	Aliases: []string{"website"},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		return "Check out the dashboard [here](" + config.BaseUri.GetString() + "/dashboard).", nil
	},
}
