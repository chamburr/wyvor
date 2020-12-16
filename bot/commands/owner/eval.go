package owner

import (
	"fmt"
	"go/build"
	"reflect"
	"strings"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/sirupsen/logrus"
	"github.com/traefik/yaegi/interp"
	"github.com/traefik/yaegi/stdlib"
)

var (
	log = logger.WithPrefix("eval")
)

var EvalCommand = &command.Command{
	Name: "eval",
	Arguments: []*command.Argument{
		{Name: "code", Type: command.ArgumentString, Required: true},
	},
	PermChecks: []command.PermFunc{
		command.BotOwner,
	},
	AllowDM: true,
	Run: func(data *command.CommandData) (res interface{}, err error) {
		code, _ := data.Arg(0)

		imports := ""
		lines := strings.Split(code.(string), "\n")
		for index, line := range lines {
			if strings.HasPrefix(line, "import ") {
				imports += line + "\n"
				code = code.(string)[len(line)+1:]
			}

			if len(lines)-1 == index && strings.HasPrefix(line, "return ") == false {
				code = code.(string) + "\nreturn nil"
			}
		}

		interpreter := interp.New(interp.Options{GoPath: build.Default.GOPATH})
		interpreter.Use(stdlib.Symbols)

		deps := make(map[string]map[string]reflect.Value)
		deps["common"] = make(map[string]reflect.Value)
		deps["common"]["Version"] = reflect.ValueOf(common.Version)
		deps["common"]["Shards"] = reflect.ValueOf(common.Shards)
		deps["common"]["Rabbit"] = reflect.ValueOf(common.Rabbit)
		deps["common"]["Redis"] = reflect.ValueOf(common.Redis)
		deps["common"]["Session"] = reflect.ValueOf(common.Session)
		deps["common"]["BotUser"] = reflect.ValueOf(common.BotUser)
		deps["common"]["State"] = reflect.ValueOf(common.State)
		deps["command"] = make(map[string]reflect.Value)
		deps["command"]["CommandData"] = reflect.ValueOf((*command.CommandData)(nil))
		deps["logrus"] = make(map[string]reflect.Value)
		deps["logrus"]["Entry"] = reflect.ValueOf((*logrus.Entry)(nil))
		interpreter.Use(deps)

		source := "package main\n\n"
		source += "import \"common\"\n"
		source += "import \"command\"\n"
		source += "import \"logrus\"\n"
		source += imports + "\n"
		source += "func Run(log *logrus.Entry, data *command.CommandData) (res interface{}) {\n"
		source += code.(string) + "\n"
		source += "}"

		_, err = interpreter.Eval(source)
		if err != nil {
			return utils.ErrorEmbed(err.Error()), nil
		}

		reflect, err := interpreter.Eval("Run")
		if err != nil {
			return utils.ErrorEmbed(err.Error()), nil
		}

		defer func() {
			if err := recover(); err != nil {
				res = utils.ErrorEmbed(fmt.Sprint(err))
				err = nil
				return
			}
		}()

		function := reflect.Interface().(func(entry *logrus.Entry, data *command.CommandData) (res interface{}))
		result := function(log, data)

		if result == nil {
			err = common.Session.MessageReactionAdd(data.Channel.ID, data.Message.ID, "✅")
			return
		}

		res, err = utils.ShortenCode(fmt.Sprint(result))
		if err != nil {
			return
		}

		return
	},
}
