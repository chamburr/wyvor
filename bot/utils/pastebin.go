package utils

import (
	"bytes"
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"io/ioutil"
	"net/http"

	"github.com/btcsuite/btcutil/base58"
	"golang.org/x/crypto/pbkdf2"
)

func ToBase64(src []byte) string {
	return base64.RawStdEncoding.EncodeToString(src)
}

func GenRandomBytes(size uint32) ([]byte, error) {
	result := make([]byte, size)
	_, err := rand.Read(result)
	return result, err
}

func Upload(input string) (string, error) {
	data, err := json.Marshal(&map[string]string{"paste": input})
	if err != nil {
		return "", err
	}

	master, err := GenRandomBytes(32)
	if err != nil {
		return "", err
	}

	iv, err := GenRandomBytes(12)
	if err != nil {
		return "", err
	}

	salt, err := GenRandomBytes(8)
	if err != nil {
		return "", err
	}

	spec := []interface{}{
		[]interface{}{
			ToBase64(iv),
			ToBase64(salt),
			100000,
			256,
			128,
			"aes",
			"gcm",
			"none",
		},
		"syntaxhighlighting",
		0,
		0,
	}

	key := pbkdf2.Key(master, salt, 100000, 32, sha256.New)

	aData, err := json.Marshal(spec)
	if err != nil {
		return "", err
	}

	aesCipher, err := aes.NewCipher(key)
	if err != nil {
		return "", err
	}

	gcmCipher, err := cipher.NewGCM(aesCipher)
	if err != nil {
		return "", err
	}

	request := map[string]interface{}{
		"v":     2,
		"adata": spec,
		"meta": map[string]interface{}{
			"expire": "1week",
		},
		"ct": ToBase64(gcmCipher.Seal(nil, iv, data, aData)),
	}

	body, err := json.Marshal(request)
	if err != nil {
		return "", err
	}

	client := &http.Client{}
	req, err := http.NewRequest("POST", "https://chamburr.xyz/p/", bytes.NewBuffer(body))
	if err != nil {
		return "", err
	}

	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	req.Header.Set("X-Requested-With", "JSONHttpRequest")

	resp, err := client.Do(req)
	if err != nil {
		return "", err
	}

	defer func() {
		_ = resp.Body.Close()
	}()

	respBody, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}

	var response map[string]interface{}

	err = json.Unmarshal(respBody, &response)
	if err != nil {
		return "", err
	}

	if response["url"] == nil {
		return "", nil
	}

	return "https://chamburr.xyz" + response["url"].(string) + "#" + base58.Encode(master), nil
}
