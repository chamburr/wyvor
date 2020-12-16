package config

import (
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/chamburr/wyvor/utils/logger"
	"github.com/joho/godotenv"
)

var (
	log = logger.WithPrefix("config")
)

var (
	BaseUri     *Option
	Environment *Option

	SentryDsn *Option

	BotToken *Option

	MainGuild     *Option
	BotOwnerRole  *Option
	BotAdminRole  *Option
	GuildsChannel *Option

	ApiHost   *Option
	ApiPort   *Option
	ApiSecret *Option

	RedisHost *Option
	RedisPort *Option

	RabbitHost *Option
	RabbitPort *Option

	PrometheusHost *Option
	PrometheusPort *Option

	PprofHost *Option
	PprofPort *Option
)

func Init() {
	err := godotenv.Load()
	if err != nil {
		log.WithError(err).Fatal("Failed to load env file")
	}

	BaseUri = register("base_uri", true)
	Environment = register("environment", true)

	SentryDsn = register("sentry_dsn", false)

	BotToken = register("bot_token", true)

	MainGuild = register("main_guild", true)
	BotOwnerRole = register("bot_owner_role", true)
	BotAdminRole = register("bot_admin_role", true)
	GuildsChannel = register("guilds_channel", true)

	ApiHost = register("api_host", true)
	ApiPort = register("api_port", true)
	ApiSecret = register("api_secret", true)

	RedisHost = register("redis_host", true)
	RedisPort = register("redis_port", true)

	RabbitHost = register("rabbit_host", true)
	RabbitPort = register("rabbit_port", true)

	PrometheusHost = register("prometheus_host", true)
	PrometheusPort = register("prometheus_port", true)

	PprofHost = register("pprof_host", true)
	PprofPort = register("pprof_port", true)

	log.Info("Configurations loaded")
}

func GetBotToken() string {
	return "Bot " + BotToken.GetString()
}

func GetApiAddress() string {
	return fmt.Sprintf("http://%s:%d/api", ApiHost.GetString(), ApiPort.GetInt())
}

func GetRedisAddress() string {
	return fmt.Sprintf("%s:%d", RedisHost.GetString(), RedisPort.GetInt())
}

func GetRabbitAddress() string {
	return fmt.Sprintf("%s:%d", RabbitHost.GetString(), RabbitPort.GetInt())
}

func GetPrometheusAddress() string {
	return fmt.Sprintf("%s:%d", PrometheusHost.GetString(), PrometheusPort.GetInt())
}

func GetPprofAddress() string {
	return fmt.Sprintf("%s:%d", PprofHost.GetString(), PprofPort.GetInt())
}

func GetDevelopment() bool {
	return strings.ToLower(Environment.GetString()) == "development"
}

func register(name string, required bool) *Option {
	name = strings.ToUpper(name)
	value := os.Getenv(name)

	if required && value == "" {
		log.Fatal("Missing required Option: " + name)
	}

	option := &Option{
		Name:     name,
		Value:    interface{}(value),
		Required: required,
	}

	return option
}

type Option struct {
	Name     string
	Value    interface{}
	Required bool
}

func (option *Option) GetString() string {
	if value, ok := option.Value.(string); ok {
		return value
	}

	return ""
}

func (option *Option) GetInt() int {
	if value, ok := option.Value.(string); ok {
		num, _ := strconv.ParseInt(value, 10, 64)
		return int(num)
	}

	return 0
}

func (option *Option) GetInt64() int64 {
	if value, ok := option.Value.(string); ok {
		num, _ := strconv.ParseInt(value, 10, 64)
		return num
	}

	return 0
}

func (option *Option) GetBool() bool {
	if value, ok := option.Value.(string); ok {
		if strings.ToLower(value) == "true" {
			return true
		}
	}

	return false
}
