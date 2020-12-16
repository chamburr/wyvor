package api

import (
	"bytes"
	"encoding/json"
	"io/ioutil"
	"net/http"

	"github.com/chamburr/wyvor/config"
	"github.com/chamburr/wyvor/utils/logger"
	"github.com/jonas747/discordgo"
)

var (
	client = &http.Client{Timeout: config.ApiTimeout}

	log = logger.WithPrefix("api")
)

type ApiMessage struct {
	Message string `json:"message"`
}

type ApiError struct {
	Request  *http.Request
	Response *http.Response
	Body     []byte
}

func (apiErr ApiError) Message() string {
	var message *ApiMessage
	err := json.Unmarshal(apiErr.Body, &message)
	if err == nil {
		return message.Message
	}

	return "HTTP error: " + apiErr.Response.Status
}

func (apiErr ApiError) Error() string {
	return apiErr.Message()
}

func RequestGet(user *discordgo.User, url string) (result map[string]interface{}, err error) {
	res, err := Request(user, "GET", url, nil)
	if err != nil {
		return nil, err
	}

	return res.(map[string]interface{}), err
}

func RequestGetArray(user *discordgo.User, url string) (result []interface{}, err error) {
	res, err := Request(user, "GET", url, nil)
	if err != nil {
		return nil, err
	}

	return res.([]interface{}), err
}

func RequestPost(user *discordgo.User, url string, data interface{}) (result map[string]interface{}, err error) {
	res, err := Request(user, "POST", url, data)
	if err != nil {
		return nil, err
	}

	return res.(map[string]interface{}), err
}

func RequestPut(user *discordgo.User, url string, data interface{}) (result map[string]interface{}, err error) {
	res, err := Request(user, "PUT", url, data)
	if err != nil {
		return nil, err
	}

	return res.(map[string]interface{}), err
}

func RequestPatch(user *discordgo.User, url string, data interface{}) (result map[string]interface{}, err error) {
	res, err := Request(user, "PATCH", url, data)
	if err != nil {
		return nil, err
	}

	return res.(map[string]interface{}), err
}

func RequestDelete(user *discordgo.User, url string) (result map[string]interface{}, err error) {
	res, err := Request(user, "DELETE", url, nil)
	if err != nil {
		return nil, err
	}

	return res.(map[string]interface{}), err
}

func Request(user *discordgo.User, method string, url string, data interface{}) (result interface{}, err error) {
	var buffer []byte
	if data != nil {
		buffer, err = json.Marshal(data)
		if err != nil {
			return
		}
	}

	url = config.GetApiAddress() + url

	req, err := http.NewRequest(method, url, bytes.NewBuffer(buffer))
	if err != nil {
		return
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", config.ApiSecret.GetString())

	req.Header.Set("User-Id", discordgo.StrID(user.ID))
	req.Header.Set("User-Username", user.Username)
	req.Header.Set("User-Discriminator", user.Discriminator)
	req.Header.Set("User-Avatar", user.Avatar)

	req.Close = true

	resp, err := client.Do(req)
	if err != nil {
		return
	}

	defer func() {
		err2 := resp.Body.Close()
		if err2 != nil {
			log.WithError(err2).Error("Failed to close response body")
		}
	}()

	body, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return
	}

	switch resp.StatusCode {
	case http.StatusOK:
	case http.StatusUnauthorized:
		log.Fatal("Api secret provided is invalid")
		break
	default:
		err = ApiError{
			Request:  req,
			Response: resp,
			Body:     body,
		}
		return
	}

	decoder := json.NewDecoder(bytes.NewReader(body))
	decoder.UseNumber()

	err = decoder.Decode(&result)
	if err != nil {
		return
	}

	return
}
