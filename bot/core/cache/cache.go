package cache

import (
	"encoding/json"
	"strconv"
	"time"

	"github.com/chamburr/wyvor/common"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/mediocregopher/radix/v3"
)

var (
	log = logger.WithPrefix("cache")
)

func Init() {
	go RunJobFlushGuilds()
	go RunJobFlushStats()
	go RunJobFlushAdmins()

	log.Info("Running background jobs")
}

func GetData(key string) (string, error) {
	var data string
	err := common.Redis.Do(radix.Cmd(&data, "GET", key))

	if len(data) > 2 && data[0] == '"' && data[len(data)-1] == '"' {
		data = data[1 : len(data)-1]
	}

	return data, err
}

func SetData(key string, data string) error {
	err := common.Redis.Do(radix.Cmd(nil, "SET", key, data))

	return err
}

func SetDataExpire(key string, data string, expire time.Duration) error {
	expiry := int(expire.Seconds())
	err := common.Redis.Do(radix.Cmd(nil, "SET", key, data, "EX", strconv.Itoa(expiry)))

	return err
}

func GetDataJson(key string, dest interface{}) error {
	data, err := GetData(key)
	if err != nil {
		return err
	}

	err = json.Unmarshal([]byte(data), &dest)
	return err
}

func SetDataJson(key string, data interface{}) error {
	rawData, err := json.Marshal(data)
	if err != nil {
		return err
	}

	return SetData(key, string(rawData))
}

func SetDataJsonExpire(key string, data interface{}, expire time.Duration) error {
	rawData, err := json.Marshal(data)
	if err != nil {
		return err
	}

	return SetDataExpire(key, string(rawData), expire)
}
