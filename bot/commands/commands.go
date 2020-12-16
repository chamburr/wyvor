package commands

import (
	"github.com/chamburr/wyvor/commands/admin"
	"github.com/chamburr/wyvor/commands/core"
	"github.com/chamburr/wyvor/commands/general"
	"github.com/chamburr/wyvor/commands/owner"
	"github.com/chamburr/wyvor/commands/player"
	"github.com/chamburr/wyvor/commands/playlist"
	"github.com/chamburr/wyvor/commands/queue"
	"github.com/chamburr/wyvor/core/command"
	"github.com/chamburr/wyvor/utils/logger"
)

var (
	log = logger.WithPrefix("commands")
)

func Init() {
	command.Register(admin.BanUserCommand)
	command.Register(admin.EchoCommand)
	command.Register(admin.FindServersCommand)
	command.Register(admin.SharedServersCommand)
	command.Register(admin.TopServersCommand)
	command.Register(admin.UnbanUserCommand)

	command.Register(general.DashboardCommand)
	command.Register(general.HelpCommand)
	command.Register(general.InviteCommand)
	command.Register(general.PingCommand)
	command.Register(general.PrefixCommand)
	command.Register(general.StatsCommand)
	command.Register(general.SupportCommand)

	command.Register(core.ConnectCommand)
	command.Register(core.DisconnectCommand)
	command.Register(core.EffectsCommand)
	command.Register(core.EqualizerCommand)
	command.Register(core.LyricsCommand)
	command.Register(core.NowPlayingCommand)
	command.Register(core.PlayCommand)
	command.Register(core.PlayFileCommand)
	command.Register(core.QueueCommand)

	command.Register(owner.BashCommand)
	command.Register(owner.DisconnectAllCommand)
	command.Register(owner.EvalCommand)
	command.Register(owner.InvokeCommand)
	command.Register(owner.ReconnectCommand)
	command.Register(owner.SetStatusCommand)
	command.Register(owner.ShutdownCommand)

	command.Register(player.ForwardCommand)
	command.Register(player.LoopCommand)
	command.Register(player.PauseCommand)
	command.Register(player.ResumeCommand)
	command.Register(player.RewindCommand)
	command.Register(player.SeekCommand)
	command.Register(player.VolumeCommand)

	command.Register(playlist.PlaylistCreateCommand)
	command.Register(playlist.PlaylistDeleteCommand)
	command.Register(playlist.PlaylistLoadCommand)
	command.Register(playlist.PlaylistsCommand)
	command.Register(playlist.PlaylistShowCommand)

	command.Register(queue.ClearCommand)
	command.Register(queue.JumpCommand)
	command.Register(queue.MoveCommand)
	command.Register(queue.NextCommand)
	command.Register(queue.PreviousCommand)
	command.Register(queue.RemoveCommand)
	command.Register(queue.ShuffleCommand)

	log.Infof("Registered %d commands", len(command.Commands))
}
