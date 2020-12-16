package cache

type Guild struct {
	ID          int64  `json:"id"`
	Name        string `json:"name"`
	Icon        string `json:"icon"`
	Owner       int64  `json:"owner"`
	MemberCount int    `json:"member_count"`
}

type Stats struct {
	Version  string `json:"version"`
	Started  string `json:"started"`
	Shards   int    `json:"shards"`
	Guilds   int    `json:"guilds"`
	Roles    int    `json:"roles"`
	Channels int    `json:"channels"`
	Members  int    `json:"members"`
	Voices   int    `json:"voices"`
}

type Status struct {
	Shard   int    `json:"shard"`
	Status  string `json:"status"`
	Session string `json:"session"`
	Latency int    `json:"latency"`
	LastAck string `json:"last_ack"`
}
