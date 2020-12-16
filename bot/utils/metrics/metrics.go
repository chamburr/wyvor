package metrics

import (
	"net/http"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

var (
	log = logger.WithPrefix("prometheus")
)

var (
	DiscordHttpRequests = promauto.NewCounterVec(prometheus.CounterOpts{
		Name: "discord_http_requests",
		Help: "Number of HTTP requests made with the Discord API",
	}, []string{"method", "endpoint", "status"})

	DiscordHttpRequestsDuration = promauto.NewHistogramVec(prometheus.HistogramOpts{
		Name: "discord_http_requests_duration",
		Help: "Duration taken for Discord HTTP requests in seconds",
	}, []string{"method", "endpoint", "status"})

	DiscordGuildEvents = promauto.NewCounterVec(prometheus.CounterOpts{
		Name: "discord_guild_events",
		Help: "Discord guild join and leave events",
	}, []string{"type"})

	StateGuilds = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "state_guilds",
		Help: "Number of guilds in the state cache",
	})

	StateRoles = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "state_roles",
		Help: "Number of roles in the state cache",
	})

	StateChannels = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "state_channels",
		Help: "Number of channels in the state cache",
	})

	StateMembers = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "state_members",
		Help: "Number of members in the state cache",
	})

	StateVoices = promauto.NewGauge(prometheus.GaugeOpts{
		Name: "state_voices",
		Help: "Number of voice connections in the state cache",
	})

	StateCacheHits = promauto.NewCounter(prometheus.CounterOpts{
		Name: "state_cache_hits",
		Help: "Number of state cache hits",
	})

	StateCacheMisses = promauto.NewCounter(prometheus.CounterOpts{
		Name: "state_cache_misses",
		Help: "Number of state cache misses",
	})

	StateCacheEvictions = promauto.NewCounter(prometheus.CounterOpts{
		Name: "state_cache_evicted",
		Help: "Number of state cache evictions",
	})

	StateCacheMemberEvictions = promauto.NewCounter(prometheus.CounterOpts{
		Name: "state_cache_members_evicted",
		Help: "Number of member state cache evictions",
	})

	PubsubEvents = promauto.NewCounterVec(prometheus.CounterOpts{
		Name: "pubsub_events",
		Help: "Events received through the Redis pubsub",
	}, []string{"type"})
)

func Init() {
	log.Infof("Listening on %s", config.GetPrometheusAddress())

	go func() {
		err := http.ListenAndServe(config.GetPrometheusAddress(), promhttp.Handler())
		if err != nil {
			log.WithError(err).Fatal("Failed to start server")
		}
	}()

	go RunJobFlushState()
}
