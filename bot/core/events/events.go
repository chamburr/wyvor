package events

import (
	"encoding/json"
	"runtime/debug"
	"strings"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/jonas747/discordgo"
	"github.com/jonas747/dstate/v2"
	"github.com/jonas747/gojay"
	"github.com/mailru/easyjson"
	"github.com/sirupsen/logrus"
)

var (
	handlers    = make(map[string][][]*Handler)
	numHandlers int

	log = logger.WithPrefix("events")
)

func Init() {
	for i := range handlers {
		handlers[i] = make([][]*Handler, 3)
	}

	InitHandlers()
}

type Handler func(event *EventData) (err error)

type EventData struct {
	Event interface{}
	Type  string

	Guild   *dstate.GuildState
	Channel *dstate.ChannelState
}

type Order int

const (
	OrderSyncPreState   Order = 0
	OrderSyncPostState  Order = 1
	OrderAsyncPostState Order = 2
)

func AddHandler(handler Handler, order Order, events ...string) {
	for _, event := range events {
		if event == EventAll {
			events = AllEvents
			break
		}
	}

	for _, event := range events {
		if _, ok := handlers[event]; !ok {
			handlers[event] = make([][]*Handler, 3)
		}

		handlers[event][int(order)] = append(handlers[event][int(order)], &handler)
	}

	numHandlers++
}

func AddHandlerFirst(handler Handler, events ...string) {
	AddHandler(handler, OrderSyncPreState, events...)
}

func AddHandlerSecond(handler Handler, events ...string) {
	AddHandler(handler, OrderSyncPostState, events...)
}

func AddHandlerAsync(handler Handler, events ...string) {
	AddHandler(handler, OrderAsyncPostState, events...)
}

func emitEvent(data *EventData, event string) {
	handler, ok := handlers[event]
	if !ok {
		return
	}

	runEvents(handler[0], data)
	runEvents(handler[1], data)

	go func() {
		defer func() {
			if err := recover(); err != nil {
				stack := string(debug.Stack())
				log.WithField(logrus.ErrorKey, err).WithField("event", data.Type).Error("Recovered from panic\n" + stack)
			}
		}()

		runEvents(handler[2], data)
	}()
}

func runEvents(handlers []*Handler, data *EventData) {
	for _, handler := range handlers {
		err := (*handler)(data)

		if err != nil {
			log.WithField("event", data.Type).WithError(err).Error("An error occurred")
		}
	}
}

func HandleEvent(event *discordgo.Event) {
	var data = &EventData{Type: event.Type}

	for _, eventName := range AllEvents {
		if event.Type == strings.ToUpper(eventName) {
			data.Event = NewEvents[eventName]()

			if decoder, ok := data.Event.(gojay.UnmarshalerJSONObject); ok {
				err := gojay.UnmarshalJSONObject(event.RawData, decoder)
				if err != nil {
					log.WithError(err).Error("Failed unmarshalling event data with gojay\n" + string(event.RawData))
					return
				}
			} else if decoder, ok := data.Event.(easyjson.Unmarshaler); ok {
				err := easyjson.Unmarshal(event.RawData, decoder)
				if err != nil {
					log.WithError(err).Error("Failed unmarshalling event data with easyjson\n" + string(event.RawData))
					return
				}
			} else {
				err := json.Unmarshal(event.RawData, data.Event)
				if err != nil {
					log.WithError(err).Error("Failed unmarshalling event data\n" + string(event.RawData))
					return
				}
			}

			break
		}
	}

	if data.Event == nil {
		log.Warn("Unknown event received: " + event.Type)
		return
	}

	go func() {
		if guild, ok := data.Event.(discordgo.GuildEvent); ok {
			data.Guild = common.State.Guild(true, guild.GetGuildID())
		}

		if channel, ok := data.Event.(discordgo.ChannelEvent); ok {
			data.Channel = common.State.Channel(true, channel.GetChannelID())
		}

		defer func() {
			if err := recover(); err != nil {
				stack := string(debug.Stack())
				log.WithField(logrus.ErrorKey, err).WithField("event", data.Type).Error("Recovered from panic\n" + stack)
			}
		}()

		emitEvent(data, strings.ToLower(data.Type))
	}()
}
