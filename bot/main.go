package main

import (
	"os"
	"os/signal"
	"syscall"

	"github.com/chamburr/wyvor/commands"
	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/core"
	"github.com/chamburr/wyvor/utils/debug"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/chamburr/wyvor/utils/metrics"
	"github.com/getsentry/sentry-go"
	log "github.com/sirupsen/logrus"
)

const (
	Version = "1.0.1"
)

func main() {
	log.Infof(`╭──────────────────────────────────────────────╮`)
	log.Infof(`│                                              │`)
	log.Infof(`│    __          __                            │`)
	log.Infof(`│    \ \        / /                            │`)
	log.Infof(`│     \ \  /\  / /_   _ __   __ ___   _ __     │`)
	log.Infof(`│      \ \/  \/ /| | | |\ \ / // _ \ | '__|    │`)
	log.Infof(`│       \  /\  / | |_| | \ V /| (_) || |       │`)
	log.Infof(`│        \/  \/   \__, |  \_/  \___/ |_|       │`)
	log.Infof(`│                  __/ |                       │`)
	log.Infof(`│                 |___/       version %-8s │`, Version)
	log.Infof(`│                                              │`)
	log.Infof(`╰──────────────────────────────────────────────╯`)
	log.Infof(``)

	common.Version = Version

	logger.Init()
	config.Init()
	core.Init()
	commands.Init()
	debug.Init()
	metrics.Init()

	if !config.GetDevelopment() {
		err := sentry.Init(sentry.ClientOptions{Dsn: config.SentryDsn.GetString()})
		if err != nil {
			logger.WithPrefix("core").WithError(err).Fatalf("Failed to initialize sentry")
		}
	}

	c := make(chan os.Signal, 2)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)

	<-c
	core.Shutdown()
}
