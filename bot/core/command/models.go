package command

import (
	"strconv"
	"strings"
	"time"

	"github.com/chamburr/wyvor/common"
	"github.com/jonas747/discordgo"
	"github.com/jonas747/dstate/v2"
)

type Command struct {
	Name      string
	Aliases   []string
	Arguments []*Argument

	PermChecks []PermFunc

	Disabled bool
	AllowDM  bool

	Run RunFunc
}

type CommandData struct {
	Command   *Command
	Arguments []*Argument

	GuildPrefix string
	Prefix      string
	Content     string

	Message *discordgo.Message
	Channel *dstate.ChannelState
	Guild   *dstate.GuildState
	Author  *discordgo.User
}

type PermFunc func(user *discordgo.User) (bool, error)

type RunFunc func(data *CommandData) (res interface{}, err error)

type Argument struct {
	Name     string
	Type     ArgumentType
	Required bool
	Value    string
}

type ArgumentType string

const (
	ArgumentString   ArgumentType = "string"
	ArgumentInt      ArgumentType = "integer"
	ArgumentBool     ArgumentType = "bool"
	ArgumentDuration ArgumentType = "duration"
	ArgumentUser     ArgumentType = "user"
)

func (data *CommandData) Arg(index int) (interface{}, error) {
	if len(data.Arguments) <= index {
		return nil, nil
	}

	argument := data.Arguments[index]

	switch argument.Type {
	case ArgumentString:
		return argument.Value, nil
	case ArgumentInt:
		return strconv.ParseInt(argument.Value, 10, 64)
	case ArgumentBool:
		value := strings.ToLower(argument.Value)
		if value == "true" || value == "yes" || value == "enable" {
			return true, nil
		} else {
			return false, nil
		}
	case ArgumentDuration:
		value := strings.SplitN(argument.Value, ":", 2)
		if len(value) == 1 {
			seconds, err := strconv.Atoi(value[0])
			if err == nil {
				return time.Duration(seconds) * time.Second, nil
			}
		} else if len(value) == 2 {
			seconds, err := strconv.Atoi(value[1])
			if err == nil {
				minutes, err := strconv.Atoi(value[0])
				if err == nil {
					return time.Duration(minutes)*time.Minute + time.Duration(seconds)*time.Second, nil
				}
			}
		}
		return time.ParseDuration(argument.Value)
	case ArgumentUser:
		value := argument.Value
		value = strings.Replace(value, "<@", "", 1)
		value = strings.Replace(value, "!", "", 1)
		value = strings.Replace(value, ">", "", 1)

		id, err := strconv.ParseInt(value, 10, 64)
		if err != nil {
			return nil, err
		}

		user, err := common.Session.User(id)
		if err != nil {
			return nil, err
		}

		return user, nil
	default:
		return nil, nil
	}
}
