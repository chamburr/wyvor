package core

import (
	"bytes"
	"encoding/gob"
	"fmt"
	"io"
	"os"
	"strconv"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core/cache"
	"github.com/chamburr/wyvor/core/events"
	"github.com/chamburr/wyvor/core/pubsub"
	"github.com/chamburr/wyvor/core/queue"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/chamburr/wyvor/utils/metrics"
	"github.com/jonas747/discordgo"
	"github.com/jonas747/dstate/v2"
	"github.com/mediocregopher/radix/v3"
	"github.com/streadway/amqp"
)

var (
	log = logger.WithPrefix("core")
)

func Init() {
	session, err := discordgo.New(config.GetBotToken())
	if err != nil {
		log.WithError(err).Fatal("Failed to create http session")
	}

	botUser, err := session.UserMe()
	if err != nil {
		log.WithError(err).Fatal("Failed getting current user")
	}

	session.MaxRestRetries = config.DiscordMaxRestRetries
	session.Ratelimiter.MaxConcurrentRequests = config.DiscordMaxCCRequests
	session.Client.Transport = metrics.NewTransport()
	session.State.User = &discordgo.SelfUser{User: botUser}

	state := dstate.NewState()
	state.TrackPresences = false
	state.KeepDeletedMessages = false
	state.TrackMessages = false
	state.CacheExpirey = config.StateCacheExpiry
	go state.RunGCWorker()

	file, err := os.Open("./cache.db")
	if err != nil {
		log.WithError(err).Warn("Not loading state cache")
	} else {
		var stateCache []*discordgo.Guild

		err := gob.NewDecoder(file).Decode(&stateCache)
		if err != nil {
			log.WithError(err).Fatal("Failed to decode state cache")
		}

		for _, guild := range stateCache {
			state.GuildCreate(true, guild)
		}

		log.Info("Loaded state cache from file")

		err = file.Close()
		if err != nil {
			log.WithError(err).Error("Failed to close cache file")
		}
	}

	common.Session = session
	common.BotUser = botUser
	common.State = state

	rabbit, err := amqp.Dial(fmt.Sprintf("amqp://%s/%%2f", config.GetRabbitAddress()))
	if err != nil {
		log.WithError(err).Fatal("Failed to connect to rabbit")
	}
	log.Info("Connected to rabbit server")

	redis, err := radix.NewPool("tcp", config.GetRedisAddress(), config.RedisMaxConnections)
	if err != nil {
		log.WithError(err).Fatal("Failed creating redis pool")
	}
	log.Info("Connected to redis database")

	common.Rabbit = rabbit
	common.Redis = redis

	shards, err := cache.GetData(config.ShardKey)
	if err != nil {
		log.WithError(err).Fatal("Failed getting number of shards")
	}

	common.Shards, err = strconv.Atoi(shards)
	if err != nil {
		log.WithError(err).Fatal("Invalid shard found")
	}
	log.Infof("Found %d shards", common.Shards)

	cache.Init()
	events.CollectEvent = metrics.CollectGatewayEvent
	events.Init()
	queue.HandleEvent = events.HandleEvent
	queue.Init()
	pubsub.CollectEvent = metrics.CollectPubsubEvent
	pubsub.Init()

	err = queue.GetGuildMembers(config.MainGuild.GetInt64())
	if err != nil {
		log.WithError(err).Error("Failed to get members for main guild")
	}
}

func Shutdown() {
	log.Info("Shutting down")

	err := common.Rabbit.Close()
	if err != nil {
		log.WithError(err).Error("Failed to close rabbit connection")
	} else {
		log.Info("Closed rabbit connection")
	}

	err = common.Redis.Close()
	if err != nil {
		log.WithError(err).Error("Failed to close redis connection")
	} else {
		log.Info("Closed redis connection")
	}

	common.State.RLock()
	var state []*discordgo.Guild
	for _, guild := range common.State.Guilds {
		guildCopy := guild.DeepCopy(false, true, true, true)
		guild.RLock()
		for _, member := range guild.Members {
			memberCopy := member.DGoCopy()
			guildCopy.Members = append(guildCopy.Members, memberCopy)
		}
		guild.RUnlock()
		state = append(state, guildCopy)
	}
	common.State.RUnlock()

	buffer := &bytes.Buffer{}
	err = gob.NewEncoder(buffer).Encode(state)
	if err != nil {
		log.WithError(err).Fatal("Failed to encode state")
	}

	file, err := os.Create("./cache.db")
	if err != nil {
		log.WithError(err).Fatal("Failed to create cache file")
	}

	_, err = io.Copy(file, buffer)
	if err != nil {
		log.WithError(err).Fatal("Failed to write to cache file")
	}

	err = file.Close()
	if err != nil {
		log.WithError(err).Fatal("Failed to close cache file")
	}

	log.Info("Saved state cache to file")

	log.Info("Shutdown complete")

	os.Exit(0)
}
