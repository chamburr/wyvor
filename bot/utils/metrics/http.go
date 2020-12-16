package metrics

import (
	"fmt"
	"net"
	"net/http"
	"regexp"
	"strconv"
	"time"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/prometheus/client_golang/prometheus"
)

var (
	regexEndpointVersion = regexp.MustCompile(`/api/v[0-9]+`)
	regexEndpointParam   = regexp.MustCompile(`/[%A-Z0-9]+`)

	httpLog = logger.WithPrefix("http")
)

func NewTransport() *LoggingTransport {
	inner := &http.Transport{
		Proxy: http.ProxyFromEnvironment,
		DialContext: (&net.Dialer{
			Timeout:   config.HttpDialTimeout,
			KeepAlive: config.HttpDialKeepAlive,
		}).DialContext,
		TLSHandshakeTimeout:   config.HttpTlsHandshakeTimeout,
		MaxIdleConns:          config.HttpMaxIdleConns,
		MaxIdleConnsPerHost:   config.HttpMaxIdleConnsPerHost,
		IdleConnTimeout:       config.HttpIdleConnTimeout,
		ExpectContinueTimeout: config.HttpExpectContinueTimeout,
	}

	return &LoggingTransport{Inner: inner}
}

type LoggingTransport struct {
	Inner http.RoundTripper
}

func (transport *LoggingTransport) RoundTrip(request *http.Request) (*http.Response, error) {
	inner := transport.Inner
	if inner == nil {
		inner = http.DefaultTransport
	}

	started := time.Now()

	code := 0
	resp, err := inner.RoundTrip(request)
	if resp != nil {
		code = resp.StatusCode
	}

	duration := time.Since(started)

	go func() {
		endpoint := request.URL.Path
		endpoint = regexEndpointVersion.ReplaceAllString(endpoint, "")
		endpoint = regexEndpointParam.ReplaceAllString(endpoint, "/:id")

		labels := prometheus.Labels{
			"method":   request.Method,
			"status":   strconv.Itoa(code),
			"endpoint": endpoint,
		}

		DiscordHttpRequests.With(labels).Inc()
		DiscordHttpRequestsDuration.With(labels).Observe(duration.Seconds())

		if duration > config.HttpLogDurationThreshold {
			route := fmt.Sprintf("%s %s", request.Method, request.URL.Path)
			httpLog.WithField("route", route).Warnf("Request took %d ms to complete", duration.Milliseconds())
		}
	}()

	return resp, err
}
