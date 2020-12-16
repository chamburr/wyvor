package pubsub

type Channel struct {
	ID       int64  `json:"id"`
	Name     string `json:"name"`
	Kind     int    `json:"kind"`
	Position int    `json:"position"`
	Parent   int64  `json:"parent"`
}

type Role struct {
	ID       int64  `json:"id"`
	Name     string `json:"name"`
	Color    int    `json:"color"`
	Position int    `json:"position"`
}

type ReceiveGetUser struct {
	User int64 `json:"user"`
}

type ReceiveGetMember struct {
	Guild  int64 `json:"guild"`
	Member int64 `json:"member"`
}

type ReceiveGetGuild struct {
	Guild int64 `json:"guild"`
}

type ReceiveGetPermission struct {
	Guild   int64 `json:"guild"`
	Member  int64 `json:"member"`
	Channel int64 `json:"channel"`
}

type ReceiveSendMessage struct {
	Channel int64  `json:"channel"`
	Title   string `json:"title"`
	Content string `json:"content"`
	Author  int64  `json:"author"`
}

type ReceiveGetConnected struct {
	Guild  int64 `json:"guild"`
	Member int64 `json:"member"`
}

type ReceiveSetConnected struct {
	Guild   int64 `json:"guild"`
	Channel int64 `json:"channel"`
}

type RespondGetUser struct {
	ID            int64  `json:"id"`
	Username      string `json:"username"`
	Discriminator int    `json:"discriminator"`
	Avatar        string `json:"avatar"`
}

type RespondGetMember struct {
	ID            int64   `json:"id"`
	Username      string  `json:"username"`
	Discriminator int     `json:"discriminator"`
	Avatar        string  `json:"avatar"`
	Nickname      string  `json:"nickname"`
	Roles         []int64 `json:"roles"`
	JoinedAt      string  `json:"joined_at"`
}

type RespondGetPermission struct {
	Permission int `json:"permission"`
}

type RespondGetGuild struct {
	ID          int64     `json:"id"`
	Name        string    `json:"name"`
	Icon        string    `json:"icon"`
	Region      string    `json:"region"`
	Owner       int64     `json:"owner"`
	MemberCount int       `json:"member_count"`
	Roles       []Role    `json:"roles"`
	Channels    []Channel `json:"channels"`
}

type RespondGetConnected struct {
	Channel int64   `json:"channel"`
	Members []int64 `json:"members"`
}

type SendVoiceUpdate struct {
	Session  string `json:"session"`
	Guild    int64  `json:"guild"`
	Endpoint string `json:"endpoint"`
	Token    string `json:"token"`
}
