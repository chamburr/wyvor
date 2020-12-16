package queue

import "github.com/jonas747/discordgo"

type QueueOp int

const (
	QueueOpSend QueueOp = iota
	QueueOpReconnect
)

type Event struct {
	Operation QueueOp       `json:"op"`
	Shard     int           `json:"shard"`
	Data      *DiscordEvent `json:"data,omitempty"`
}

type DiscordEvent struct {
	Operation discordgo.GatewayOP `json:"op"`
	Data      interface{}         `json:"d,omitempty"`
}

type RequestGuildMembers struct {
	GuildID string `json:"guild_id"`
	Query   string `json:"query"`
	Limit   int    `json:"limit"`
}

type UpdateVoiceState struct {
	GuildID   string      `json:"guild_id"`
	ChannelID interface{} `json:"channel_id"`
	SelfMute  bool        `json:"self_mute"`
	SelfDeaf  bool        `json:"self_deaf"`
}

type UpdateStatus struct {
	IdleSince int             `json:"since"`
	Game      *discordgo.Game `json:"game"`
	AFK       bool            `json:"afk"`
	Status    string          `json:"status"`
}
