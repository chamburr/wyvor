package command

import (
	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/jonas747/discordgo"
	"github.com/mediocregopher/radix/v3"
)

func BotAdmin(user *discordgo.User) (result bool, err error) {
	err = common.Redis.Do(radix.Cmd(&result, "SISMEMBER", config.BotAdminKey, discordgo.StrID(user.ID)))
	return
}

func BotOwner(user *discordgo.User) (result bool, err error) {
	err = common.Redis.Do(radix.Cmd(&result, "SISMEMBER", config.BotOwnerKey, discordgo.StrID(user.ID)))
	return
}
