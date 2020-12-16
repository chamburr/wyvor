# General

## Bot Prefix

-   Key: prefix
-   Type: input
-   Required: true
-   Placeholder: Enter prefix...

## Max Queue Length

-   Key: max_queue
-   Type: input-number
-   Required: true
-   Tooltip: Maximum number of tracks in the queue.
-   Placeholder: Enter max length...

## Prevent Duplicates

-   Key: no_duplicate
-   Type: select
-   Required: true
-   Options: enable
-   Tooltip: Whether to prevent duplicated tracks in the queue.

## 24/7 Mode

-   Key: keep_alive
-   Type: select
-   Required: true
-   Options: enable
-   Tooltip: Whether to have the bot always stay in the voice channel.

## Permissions

## Manage Server

-   Key: guild_roles
-   Type: select-multiple
-   Options: roles
-   Tooltip: Access bot settings and override all other permissions below.

## Manage Playlist

-   Key: playlist_roles
-   Type: select-multiple
-   Options: roles
-   Tooltip: Access to creating and deleting playlists in this server.

## Manage Player

-   Key: player_roles
-   Type: select-multiple
-   Options: roles
-   Tooltip: Access to controlling the player (such as skipping and setting effects)

## Manage Queue

-   Key: queue_roles
-   Type: select-multiple
-   Options: roles
-   Tooltip: Access to controlling the queue (such as removing tracks and clearing the queue)

## Add to Queue

-   Key: track_roles
-   Type: select-multiple
-   Options: roles
-   Tooltip: Access to adding tracks to the queue.

# Logs

## Now Playing

-   Key: playing_log
-   Type: select
-   Options: channels
-   Tooltip: Announce the currently playing track to this channel.

## Player Controls

-   Key: player_log
-   Type: select
-   Options: channels
-   Tooltip: Log player controls from the dashboard (such as skipping and setting effects) to this
    channel.

## Queue controls

-   Key: queue_log
-   Type: select
-   Options: channels
-   Tooltip: Log queue controls from the dashboard (such as adding and removing tracks) to this
    channel.
