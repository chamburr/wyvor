package command

import (
	"fmt"
	"github.com/mediocregopher/radix/v3"
	"sort"
	"strings"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/cache"
	"github.com/chamburr/wyvor/utils"
	"github.com/iancoleman/strcase"
	"github.com/jonas747/discordgo"
	"github.com/jonas747/dstate/v2"
)

var (
	Commands []*Command
)

func Register(command *Command) {
	Commands = append(Commands, command)
}

func HandleCommand(guild *dstate.GuildState, channel *dstate.ChannelState, message *discordgo.Message) (err error) {
	data := CommandData{
		Message: message,
		Channel: channel,
		Guild:   guild,
		Author:  message.Author,
	}

	embed, err := handleCommand(&data)
	if err != nil {
		return
	}

	if embed != nil {
		_, err = common.Session.ChannelMessageSendEmbed(channel.ID, embed)
		if err != nil {
			return
		}
	}

	return
}

func handleCommand(data *CommandData) (embed *discordgo.MessageEmbed, err error) {
	err = fillData(data)
	if err != nil {
		panic(err)
	}

	if data.Command == nil {
		return
	}

	if !utils.BotHasPermission(data.Guild, data.Channel, discordgo.PermissionSendMessages) {
		return
	}

	if !utils.BotHasPermission(data.Guild, data.Channel, discordgo.PermissionEmbedLinks) {
		_, err = common.Session.ChannelMessageSend(data.Channel.ID, "The **Embed Links** permission is required for basic commands.")
		return
	}

	var banned bool
	err = common.Redis.Do(radix.Cmd(&banned, "SISMEMBER", config.BlacklistKey, discordgo.StrID(data.Author.ID)))
	if err != nil {
		return
	}

	if banned == true {
		return utils.ErrorEmbed("You are banned from the bot."), nil
	}

	if data.Command.Disabled {
		return utils.ErrorEmbed("This command is currently disabled."), nil
	}

	if data.Command.AllowDM == false && data.Guild == nil {
		return utils.ErrorEmbed("This command cannot be used in Direct Message."), nil
	}

	for _, permCheck := range data.Command.PermChecks {
		pass, err := permCheck(data.Message.Author)
		if err != nil {
			return nil, err
		}

		if pass == false {
			return utils.ErrorEmbed("You do not have permission to perform this action."), nil
		}
	}

	for index, argument := range data.Command.Arguments {
		if len(data.Arguments) <= index {
			if argument.Required == true {
				return utils.ErrorEmbed("The required argument **" + argument.Name + "** is missing."), nil
			}

			break
		}

		if _, err := data.Arg(index); err != nil {
			return utils.ErrorEmbed("The argument **" + argument.Name + "** must be a " + string(argument.Type) + "."), nil
		}
	}

	result, err := data.Command.Run(data)

	if err != nil {
		switch err.(type) {
		case api.ApiError:
			embed = utils.ErrorEmbed(err.Error())
			break
		default:
			return
		}

		return embed, nil
	}

	switch result.(type) {
	case string:
		embed = utils.DefaultEmbed(data.Guild)
		embed.Title = strings.Title(strcase.ToDelimited(data.Command.Name, ' '))
		embed.Description = result.(string)
		break
	case *discordgo.MessageEmbed:
		embed = result.(*discordgo.MessageEmbed)
		break
	}

	return
}

func fillData(data *CommandData) (err error) {
	if data.Message.Author.Bot == true {
		return
	}

	prefixes := []string{common.BotUser.Mention(), strings.ReplaceAll(common.BotUser.Mention(), "@", "@!")}

	if data.Guild != nil {
		guildPrefix, err := cache.GetData(fmt.Sprintf(config.GuildPrefixKey, data.Guild.ID))
		if err != nil {
			return err
		}

		if guildPrefix == "" {
			resp, err := api.RequestGet(data.Author, api.EndpointGuildSettings(data.Guild.ID))
			if err != nil {
				return err
			}

			guildPrefix = resp["prefix"].(string)
		}

		data.GuildPrefix = guildPrefix
		prefixes = append([]string{guildPrefix}, prefixes...)
	} else {
		for char := 33; char <= 126; char++ {
			prefixes = append(prefixes, string(rune(char)))
		}
	}

	for _, prefix := range prefixes {
		if strings.HasPrefix(data.Message.Content, prefix) {
			data.Prefix = prefix
			data.Content = strings.TrimSpace(data.Message.Content[len(prefix):])
			break
		}
	}

	if data.Prefix == "" || data.Content == "" {
		return
	}

	commands := Commands
	sort.Slice(commands, func(i, j int) bool {
		return len(commands[i].Name) > len(commands[j].Name)
	})

	commandName := ""

	for _, command := range commands {
		if strings.HasPrefix(strings.ToLower(data.Content+" "), strings.ReplaceAll(command.Name, "_", "")+" ") {
			data.Command = command
			commandName = command.Name
			break
		}

		for _, alias := range command.Aliases {
			if strings.HasPrefix(strings.ToLower(data.Content+" "), strings.ReplaceAll(alias, "_", "")+" ") {
				data.Command = command
				commandName = alias
				break
			}
		}

		if data.Command != nil {
			break
		}
	}

	if data.Command == nil {
		return
	}

	if len(data.Command.Arguments) > 0 {
		index := -1
		arguments := strings.Split(data.Content, " ")[len(strings.Split(commandName, " ")):]

		for _, argument := range arguments {
			if argument == "" {
				continue
			}

			index += 1
			arg := *data.Command.Arguments[index]

			if index == len(data.Command.Arguments)-1 {
				arg.Value = strings.TrimSpace(strings.Join(arguments[index:], " "))
				data.Arguments = append(data.Arguments, &arg)
				break
			}

			arg.Value = argument
			data.Arguments = append(data.Arguments, &arg)
		}
	}

	return nil
}
