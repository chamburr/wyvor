package queue

import (
	"encoding/json"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/utils"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/jonas747/discordgo"
	"github.com/jonas747/gojay"
	"github.com/streadway/amqp"
)

var (
	HandleEvent func(event *discordgo.Event)

	channel     *amqp.Channel
	channelSend *amqp.Channel

	log = logger.WithPrefix("queue")
)

func Init() {
	var err error
	channel, err = common.Rabbit.Channel()
	if err != nil {
		log.WithError(err).Fatal("Failed to create channel")
	}

	channelSend, err = common.Rabbit.Channel()
	if err != nil {
		log.WithError(err).Fatal("Failed to create channel")
	}

	_, err = channel.QueueDeclare(config.RabbitQueueReceive, false, false, false, false, nil)
	if err != nil {
		log.WithError(err).Fatal("Failed to declare receive queue")
	}

	_, err = channelSend.QueueDeclare(config.RabbitQueueSend, false, false, false, false, nil)
	if err != nil {
		log.WithError(err).Fatal("Failed to declare send queue")
	}

	messages, err := channel.Consume(config.RabbitQueueReceive, "", true, false, false, false, nil)
	if err != nil {
		log.WithError(err).Fatal("Failed to create consumer")
	}

	go func() {
		for data := range messages {
			event := &discordgo.Event{}
			err := gojay.UnmarshalJSONObject(data.Body, event)
			if err != nil {
				log.WithError(err).Error("Failed unmarshalling event\n" + string(data.Body))
				continue
			}

			HandleEvent(event)
		}
	}()
}

func Send(data *Event) error {
	rawData, err := json.Marshal(data)
	if err != nil {
		return err
	}

	return channelSend.Publish("", config.RabbitQueueSend, false, false, amqp.Publishing{
		ContentType: "text/plain",
		Body:        rawData,
	})
}

func Reconnect(shard int) error {
	return Send(&Event{
		Operation: QueueOpReconnect,
		Shard:     shard,
	})
}

func GetGuildMembers(guild int64) error {
	return Send(&Event{
		Operation: QueueOpSend,
		Shard:     utils.ShardForGuild(guild),
		Data: &DiscordEvent{
			Operation: discordgo.GatewayOPRequestGuildMembers,
			Data: &RequestGuildMembers{
				GuildID: discordgo.StrID(guild),
				Query:   "",
				Limit:   0,
			},
		},
	})
}

func SetVoiceState(guild int64, channel int64) error {
	var channelId interface{}
	channelId = discordgo.StrID(channel)

	if channelId == "0" {
		channelId = nil
	}

	return Send(&Event{
		Operation: QueueOpSend,
		Shard:     utils.ShardForGuild(guild),
		Data: &DiscordEvent{
			Operation: discordgo.GatewayOPVoiceStateUpdate,
			Data: &UpdateVoiceState{
				GuildID:   discordgo.StrID(guild),
				ChannelID: channelId,
				SelfMute:  false,
				SelfDeaf:  false,
			},
		},
	})
}

func SetStatus(statusType discordgo.GameType, content string) error {
	for shard := 0; shard < common.Shards; shard++ {
		err := Send(&Event{
			Operation: QueueOpSend,
			Shard:     shard,
			Data: &DiscordEvent{
				Operation: discordgo.GatewayOPStatusUpdate,
				Data: &UpdateStatus{
					Game: &discordgo.Game{
						Name: content,
						Type: statusType,
					},
				},
			},
		})
		if err != nil {
			return err
		}
	}

	return nil
}
