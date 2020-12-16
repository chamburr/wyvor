package pubsub

import (
	"net/http"
	"strconv"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/core/queue"
	"github.com/chamburr/wyvor/utils"
	"github.com/jonas747/discordgo"
	"github.com/jonas747/dstate/v2"
)

var (
	CollectEvent func(event *EventData) (err error)
)

func InitHandlers() {
	AddHandler(CollectEvent, EventAll)
	AddHandler(HandleGetUser, EventGetUser)
	AddHandler(HandleGetMember, EventGetMember)
	AddHandler(HandleGetGuild, EventGetGuild)
	AddHandler(HandleGetPermission, EventGetPermission)
	AddHandler(HandleSendMessage, EventSendMessage)
	AddHandler(HandleGetConnected, EventGetConnected)
	AddHandler(HandleSetConnected, EventSetConnected)

	log.Infof("Listening to %d events", numHandlers)
}

func HandleGetUser(event *EventData) (err error) {
	data := event.GetUser()
	guilds := common.State.GuildsSlice(true)

	for _, guild := range guilds {
		if member := guild.MemberCopy(true, data.User); member != nil {
			return event.Respond(RespondGetUser{
				ID:            member.ID,
				Username:      member.Username,
				Discriminator: int(member.Discriminator),
				Avatar:        member.StrAvatar(),
			})
		}
	}

	user, err := common.Session.User(data.User)
	if err != nil && err.(*discordgo.RESTError).Response.StatusCode != http.StatusNotFound {
		return err
	}

	if user != nil {
		discriminator, _ := strconv.ParseInt(user.Discriminator, 10, 64)
		return event.Respond(RespondGetUser{
			ID:            user.ID,
			Username:      user.Username,
			Discriminator: int(discriminator),
			Avatar:        user.Avatar,
		})
	}

	return event.RespondEmpty()
}

func fetchMember(guild *dstate.GuildState, id int64) error {
	member, err := common.Session.GuildMember(guild.ID, id)
	if err != nil {
		return err
	}

	guild.MemberAddUpdate(true, member)
	return nil
}

func HandleGetMember(event *EventData) (err error) {
	data := event.GetMember()

	if guild := common.State.Guild(true, data.Guild); guild != nil {
		if member := guild.MemberCopy(true, data.Member); member != nil {
			roles := member.Roles
			if roles == nil {
				roles = []int64{guild.ID}
			}

			return event.Respond(RespondGetMember{
				ID:            member.ID,
				Username:      member.Username,
				Discriminator: int(member.Discriminator),
				Avatar:        member.StrAvatar(),
				Nickname:      member.Nick,
				Roles:         roles,
				JoinedAt:      utils.FormatTime(member.JoinedAt),
			})
		}

		err = fetchMember(guild, data.Member)
		if err != nil {
			return event.RespondEmpty()
		}

		return HandleGetMember(event)
	}

	return event.RespondEmpty()
}

func HandleGetPermission(event *EventData) (err error) {
	data := event.GetPermission()

	if guild := common.State.Guild(true, data.Guild); guild != nil {
		if member := guild.MemberCopy(true, data.Member); member != nil {
			permissions, err := guild.MemberPermissionsMS(true, data.Channel, member)

			if err != nil && err != dstate.ErrChannelNotFound {
				return err
			}

			return event.Respond(RespondGetPermission{
				Permission: permissions,
			})
		}

		err := fetchMember(guild, data.Member)
		if err != nil {
			return HandleGetPermission(event)
		}
	}

	return event.RespondEmpty()
}

func HandleGetGuild(event *EventData) (err error) {
	data := event.GetGuild()

	if guild := common.State.Guild(true, data.Guild); guild != nil {
		roles := guild.Guild.Roles
		channels := guild.Channels

		guild := RespondGetGuild{
			ID:          guild.ID,
			Name:        guild.Guild.Name,
			Icon:        guild.Guild.Icon,
			Region:      guild.Guild.Region,
			Owner:       guild.Guild.OwnerID,
			MemberCount: guild.Guild.MemberCount,
		}

		for _, role := range roles {
			role := Role{
				ID:       role.ID,
				Name:     role.Name,
				Color:    role.Color,
				Position: role.Position,
			}

			guild.Roles = append(guild.Roles, role)
		}

		for _, channel := range channels {
			channel := Channel{
				ID:       channel.ID,
				Name:     channel.Name,
				Kind:     int(channel.Type),
				Position: channel.Position,
				Parent:   channel.ParentID,
			}

			guild.Channels = append(guild.Channels, channel)
		}

		return event.Respond(guild)
	}

	return event.RespondEmpty()
}

func HandleSendMessage(event *EventData) (err error) {
	data := event.SendMessage()

	if channel := common.State.Channel(true, data.Channel); channel != nil {
		embed := utils.DefaultEmbed(channel.Guild)
		embed.Title = data.Title
		embed.Description = data.Content

		if guild := common.State.Guild(true, channel.Guild.ID); guild != nil {
			if author := guild.Member(true, data.Author); author != nil {
				embed.Author = &discordgo.MessageEmbedAuthor{
					Name:    author.DGoUser().String(),
					IconURL: author.DGoUser().AvatarURL(""),
				}
			}
		}

		_, err = common.Session.ChannelMessageSendEmbed(channel.ID, embed)
		if err != nil {
			return err
		}
	}

	return event.RespondEmpty()
}

func HandleGetConnected(event *EventData) (err error) {
	data := event.GetConnected()

	if guild := common.State.Guild(true, data.Guild); guild != nil {
		if data.Member == 0 {
			data.Member = common.BotUser.ID
		}

		if state := guild.VoiceState(true, data.Member); state != nil {
			var members []int64
			for _, voice := range guild.Guild.VoiceStates {
				if voice.ChannelID == state.ChannelID {
					members = append(members, voice.UserID)
				}
			}

			return event.Respond(RespondGetConnected{
				Channel: state.ChannelID,
				Members: members,
			})
		}
	}

	return event.RespondEmpty()
}

func HandleSetConnected(event *EventData) (err error) {
	data := event.SetConnected()

	if guild := common.State.Guild(true, data.Guild); guild != nil {
		if data.Channel == 0 {
			err = queue.SetVoiceState(guild.ID, 0)
			if err != nil {
				return
			}

			return event.RespondEmpty()
		}

		if channel := guild.ChannelCopy(true, data.Channel); channel != nil {
			err = queue.SetVoiceState(guild.ID, channel.ID)
			if err != nil {
				return
			}
		}
	}

	return event.RespondEmpty()
}
