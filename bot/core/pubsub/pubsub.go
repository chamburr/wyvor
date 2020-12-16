package pubsub

import (
	"encoding/json"
	"runtime/debug"
	"strings"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/mediocregopher/radix/v3"
	"github.com/sirupsen/logrus"
)

var (
	handlers    = make(map[string][]*Handler)
	numHandlers = 0

	log = logger.WithPrefix("pubsub")
)

type EventData struct {
	Type string      `json:"op"`
	ID   string      `json:"id,omitempty"`
	Data interface{} `json:"data"`
}

type Handler func(event *EventData) (err error)

func Init() {
	conn, err := radix.PersistentPubSubWithOpts("tcp", config.GetRedisAddress())
	if err != nil {
		log.WithError(err).Fatal("Failed to connect")
	}

	log.Info("Connected to pubsub")

	messages := make(chan radix.PubSubMessage)
	if err := conn.Subscribe(messages, config.RedisPubsubChannel); err != nil {
		log.WithError(err).Fatal("Failed to subscribe to channel")
	}

	go func() {
		for message := range messages {
			HandleEvent(string(message.Message))
		}

		log.Fatal("Channel subscription ended")
	}()

	InitHandlers()
}

func AddHandler(handler Handler, events ...string) {
	for _, event := range events {
		if event == EventAll {
			events = AllEvents
		}
	}

	for _, event := range events {
		handlers[event] = append(handlers[event], &handler)
	}

	numHandlers++
}

func Publish(event string, data interface{}) error {
	eventData := EventData{
		Type: event,
		Data: data,
	}

	rawEventData, err := json.Marshal(eventData)
	if err != nil {
		return err
	}

	return common.Redis.Do(radix.Cmd(nil, "PUBLISH", config.RedisPubsubChannel, string(rawEventData)))
}

func (event *EventData) Respond(data interface{}) error {
	eventData := EventData{
		Type: EventResponse,
		ID:   event.ID,
		Data: data,
	}

	rawEventData, err := json.Marshal(eventData)
	if err != nil {
		return err
	}

	return common.Redis.Do(radix.Cmd(nil, "PUBLISH", config.RedisPubsubChannel, string(rawEventData)))
}

func (event *EventData) RespondEmpty() error {
	return event.Respond(nil)
}

func HandleEvent(message string) {
	var event EventData

	decoder := json.NewDecoder(strings.NewReader(message))
	decoder.UseNumber()

	err := decoder.Decode(&event)
	if err != nil {
		log.WithError(err).Error("Failed unmarshalling event\n" + message)
		return
	}

	go func() {
		defer func() {
			if err := recover(); err != nil {
				stack := string(debug.Stack())
				log.WithField(logrus.ErrorKey, err).WithField("event", event.Type).Error("Recovered from panic\n" + stack)
			}
		}()

		for _, handler := range handlers[event.Type] {
			err := (*handler)(&event)

			if err != nil {
				log.WithField("event", event.Type).WithError(err).Error("An error occurred")
			}
		}
	}()
}
