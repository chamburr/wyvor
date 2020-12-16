package common

import (
	"github.com/jonas747/discordgo"
	"github.com/jonas747/dstate/v2"
	"github.com/mediocregopher/radix/v3"
	"github.com/streadway/amqp"
)

var (
	Version string
	Shards  int

	Rabbit *amqp.Connection
	Redis  *radix.Pool

	Session *discordgo.Session
	BotUser *discordgo.User
	State   *dstate.State
)
