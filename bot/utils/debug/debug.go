package debug

import (
	"net/http"
	_ "net/http/pprof"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/utils/logger"
)

var (
	log = logger.WithPrefix("debug")
)

func Init() {
	log.Infof("Listening on %s", config.GetPprofAddress())

	go func() {
		err := http.ListenAndServe(config.GetPprofAddress(), nil)
		if err != nil {
			log.WithError(err).Fatal("Failed to start server")
		}
	}()
}
