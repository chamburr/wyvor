package events

import (
	"fmt"
	"time"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/api"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/core/pubsub"
	"github.com/chamburr/wyvor/utils"
	"github.com/jonas747/discordgo"
)

var (
	CollectEvent func(event *EventData) (err error)
)

func InitHandlers() {
	AddHandlerFirst(HandleReady, EventReady)
	AddHandlerFirst(HandleGuildCreate, EventGuildCreate)
	AddHandlerSecond(StateHandler, EventAll)

	AddHandlerAsync(HandleGuildDelete, EventGuildDelete)
	AddHandlerAsync(HandleMessageCreate, EventMessageCreate)
	AddHandlerAsync(HandleVoiceStateUpdate, EventVoiceStateUpdate)
	AddHandlerAsync(HandleVoiceServerUpdate, EventVoiceServerUpdate)

	log.Infof("Listening to %d events", numHandlers)
}

func HandleReady(event *EventData) (err error) {
	ready := event.Ready()

	common.Session.State.Lock()
	common.Session.State.Ready = discordgo.Ready{
		Version:   ready.Version,
		SessionID: ready.SessionID,
		User:      ready.User,
	}
	common.Session.State.Unlock()

	return
}

func StateHandler(event *EventData) (err error) {
	common.State.HandleEvent(nil, event.Event)

	return
}

func HandleGuildCreate(event *EventData) (err error) {
	guild := event.GuildCreate()

	if common.State.Guild(true, guild.ID) != nil {
		return
	}

	embed := &discordgo.MessageEmbed{
		Title:       "Guild Join",
		Description: fmt.Sprintf("%s (%d)", guild.Name, guild.ID),
		Color:       config.EmbedSuccessColor,
		Timestamp:   time.Now().Format(time.RFC3339),
		Footer: &discordgo.MessageEmbedFooter{
			Text: fmt.Sprintf("%d servers", len(common.State.Guilds)+1),
		},
	}

	_, err = common.Session.ChannelMessageSendEmbed(config.GuildsChannel.GetInt64(), embed)
	if err != nil {
		return
	}

	for _, channel := range guild.Channels {
		if channel.Type == discordgo.ChannelTypeGuildText {
			embed := utils.DefaultEmbed(nil)
			embed.Title = "Thanks for inviting " + common.BotUser.Username + "!"
			embed.Description = "The default prefix is `.`. To get started, join a voice channel and type `.play <query>`.\n\n"
			embed.Description += "View the full list of commands [here](" + config.BaseUri.GetString() + "/commands).\n\n"
			embed.Description += "Check out the dashboard for the best experience [here](" + config.BaseUri.GetString() + "/dashboard).\n\n"
			embed.Description += "Need further help? Join our support server [here](" + config.BaseUri.GetString() + "/support).\n\n"
			embed.Description += "By using " + common.BotUser.Username + ", you agree to our [Terms of Service](" + config.BaseUri.GetString() + "/terms)."

			_, err = common.Session.ChannelMessageSendEmbed(channel.ID, embed)
			if err == nil {
				break
			}
		}
	}

	err = CollectEvent(event)

	return
}

func HandleGuildDelete(event *EventData) (err error) {
	guild := event.GuildDelete()

	embed := &discordgo.MessageEmbed{
		Title:       "Guild Leave",
		Description: fmt.Sprintf("%s (%d)", guild.Name, guild.ID),
		Color:       config.EmbedErrorColor,
		Timestamp:   time.Now().Format(time.RFC3339),
		Footer: &discordgo.MessageEmbedFooter{
			Text: fmt.Sprintf("%d servers", len(common.State.Guilds)),
		},
	}

	_, err = common.Session.ChannelMessageSendEmbed(config.EmbedErrorColor, embed)

	err = CollectEvent(event)

	return
}

func HandleVoiceStateUpdate(event *EventData) (err error) {
	update := event.VoiceStateUpdate()

	if update.UserID == common.BotUser.ID {
		return
	}

	if guild := common.State.Guild(true, update.GuildID); guild != nil {
		if voice := guild.VoiceState(true, common.BotUser.ID); voice != nil {
			if update.ChannelID == 0 {
				if utils.VoiceConnections(guild, voice.ChannelID) > 1 {
					return
				}

				_, err = api.RequestPatch(common.BotUser, api.EndpointGuildPlayer(guild.ID), map[string]interface{}{
					"paused": true,
				})
				if err != nil {
					return nil
				}

				settings, err := api.RequestGet(common.BotUser, api.EndpointGuildSettings(guild.ID))
				if err != nil {
					return err
				}

				if settings["keep_alive"].(bool) == true {
					return nil
				}

				time.Sleep(config.VoiceChannelTimeout)

				if utils.VoiceConnections(guild, voice.ChannelID) > 1 {
					return nil
				}

				_, err = api.RequestDelete(common.BotUser, api.EndpointGuildPlayer(guild.ID))
				if err != nil {
					return nil
				}
			} else if update.ChannelID == voice.ChannelID {
				if utils.VoiceConnections(guild, voice.ChannelID) != 2 {
					return
				}

				player, err := api.RequestGet(common.BotUser, api.EndpointGuildPlayer(guild.ID))
				if err != nil {
					return nil
				}

				if player["paused"].(bool) == false {
					return nil
				}

				_, err = api.RequestPatch(common.BotUser, api.EndpointGuildPlayer(guild.ID), map[string]interface{}{
					"paused": false,
				})
				if err != nil {
					return nil
				}
			}
		}
	}

	return
}

func HandleMessageCreate(event *EventData) (err error) {
	message := event.MessageCreate()

	err = command.HandleCommand(event.Guild, event.Channel, message.Message)

	return
}

func HandleVoiceServerUpdate(event *EventData) (err error) {
	update := event.VoiceServerUpdate()

	if state := utils.WaitConnect(event.Guild); state != nil {
		voiceUpdate := pubsub.SendVoiceUpdate{
			Session:  state.SessionID,
			Guild:    update.GuildID,
			Endpoint: update.Endpoint,
			Token:    update.Token,
		}

		err = pubsub.Publish(pubsub.EventVoiceUpdate, voiceUpdate)
		if err != nil {
			return
		}
	}

	return
}
