package metrics

import (
	"github.com/chamburr/wyvor/core/events"
	"github.com/prometheus/client_golang/prometheus"
)

func CollectGatewayEvent(event *events.EventData) (err error) {
	switch event.Type {
	case events.EventGuildCreate:
		DiscordGuildEvents.With(prometheus.Labels{"type": "Join"}).Inc()
		break
	case events.EventGuildDelete:
		DiscordGuildEvents.With(prometheus.Labels{"type": "Leave"}).Inc()
		break
	}

	return nil
}
