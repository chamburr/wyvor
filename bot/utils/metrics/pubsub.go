package metrics

import (
	"github.com/chamburr/wyvor/core/pubsub"
	"github.com/iancoleman/strcase"
	"github.com/prometheus/client_golang/prometheus"
)

func CollectPubsubEvent(event *pubsub.EventData) (err error) {
	eventType := strcase.ToScreamingSnake(event.Type)

	PubsubEvents.With(prometheus.Labels{
		"type": eventType,
	}).Inc()

	return nil
}
