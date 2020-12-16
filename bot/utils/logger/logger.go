package logger

import (
	"context"
	"fmt"
	stdlog "log"
	"path/filepath"
	"runtime"
	"strconv"
	"strings"

	"github.com/getsentry/sentry-go"
	"github.com/jonas747/discordgo"
	log "github.com/sirupsen/logrus"
)

func Init() {
	discordgo.Logger = DiscordLogger

	stdlog.SetOutput(&STDLogger{})
	stdlog.SetFlags(0)

	log.SetFormatter(&log.TextFormatter{
		ForceColors: true,
	})

	log.AddHook(ContextHook{})
	log.AddHook(SentryHook{})
}

func getStack(skip int) string {
	pc := make([]uintptr, 3)
	runtime.Callers(skip, pc)

	fn := runtime.FuncForPC(pc[0] - 1)
	name := fn.Name()
	file, line := fn.FileLine(pc[0] - 1)
	return filepath.Base(name) + ":" + filepath.Base(file) + ":" + strconv.Itoa(line)
}

type ContextHook struct{}

func (hook ContextHook) Levels() []log.Level {
	return log.AllLevels
}

func (hook ContextHook) Fire(entry *log.Entry) error {
	i := 0

	if ctx := entry.Context; ctx != nil {
		if prefix := ctx.Value("prefix"); prefix != nil {
			entry.Message = prefix.(string) + entry.Message
		}
	}

	for _, ok := entry.Data["stack"]; !ok; {
		stack := getStack(i)

		if strings.Contains(stack, "logrus") == false {
			entry.Data["stack"] = stack
			break
		}

		i++
	}

	return nil
}

type SentryHook struct{}

func (hook SentryHook) Levels() []log.Level {
	return log.AllLevels
}

func (hook SentryHook) Fire(entry *log.Entry) error {
	exception, ok := entry.Data[log.ErrorKey].(error)

	if !ok || exception == nil {
		return nil
	}

	sentry.WithScope(func(scope *sentry.Scope) {
		scope.AddEventProcessor(func(event *sentry.Event, hint *sentry.EventHint) *sentry.Event {
			event.Message = entry.Message
			return event
		})

		scope.SetLevel(map[log.Level]sentry.Level{
			log.PanicLevel: sentry.LevelFatal,
			log.FatalLevel: sentry.LevelFatal,
			log.ErrorLevel: sentry.LevelError,
			log.WarnLevel:  sentry.LevelWarning,
			log.InfoLevel:  sentry.LevelInfo,
		}[entry.Level])

		sentry.CaptureException(exception)
	})

	return nil
}

type STDLogger struct{}

func (logger *STDLogger) Write(b []byte) (n int, err error) {
	stack := getStack(5)

	message := string(b)

	f := log.WithField("stack", stack)

	if strings.Contains(strings.ToLower(message), "error") {
		f.Error(message)
	} else {
		f.Info(message)
	}

	return len(b), err
}

func DiscordLogger(level int, caller int, message string, args ...interface{}) {
	stack := getStack(caller + 3)
	message = strings.TrimSpace(message)

	if message == "called" {
		return
	}

	f := WithPrefix("discord").WithField("stack", stack)

	switch level {
	case 0:
		f.Errorf(message, args...)
	case 1:
		f.Warnf(message, args...)
	default:
		f.Infof(message, args...)
	}
}

func WithPrefix(prefix string) *log.Entry {
	prefix = strings.ToUpper(prefix)
	prefix = fmt.Sprintf("[%s] ", prefix)
	return log.WithContext(context.WithValue(context.Background(), "prefix", prefix))
}
