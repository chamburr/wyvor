# Wyvor

[![License](https://img.shields.io/github/license/chamburr/wyvor.svg)](LICENSE)
[![CircleCI](https://circleci.com/gh/chamburr/wyvor.svg?style=shield)](https://circleci.com/gh/chamburr/wyvor)
[![Discord](https://discordapp.com/api/guilds/635412327134658571/embed.png)](https://wyvor.xyz/support)

The best feature-rich Discord music bot. Take control over your music with an intuitive dashboard,
custom effects and more!

Links:

-   [Website](https://wyvor.xyz)
-   [Bot Invite](https://wyvor.xyz/invite)
-   [Support Server](https://wyvor.xyz/support)

## Infrastructure

Wyvor has a very complex infrastructure in order to guarantee 100% uptime. Some of the
specifications are briefly explained here, please check out the code yourself for more information.

### twilight-dipatch

We use a customised gateway implementation, known as twilight-dispatch, and integrate it with
discordgo. As the gateway logic is separate from the application layer, we are able to perform
seamless upgrades and restarts in no time. Learn more
[here](https://github.com/chamburr/twilight-dispatch).

### Andesite

Andesite is a standalone node for sending audio to Discord. As opposed to the most commonly used
Lavalink, Andesite is able to provide us with much better flexibilities, such as filters support,
REST API, and better state handling. Learn more [here](https://github.com/natanbc/andesite).

### Databases

PostgreSQL, an extremely advanced and reliable relational database, is used for storing persistent
data. Redis, an in-memory data structure store, is used for caching and inter-process communication.
Prometheus is also used for metrics collection.

### RabbitMQ

RabbitMQ is used as a message broker between the gateway and the bot application layer. It is very
reliable and ensures data safety, such that events are never lost even when there is a fault.

### Rust

Rust is one of, if not, the fastest languages in the world. Thanks to the nature of this language,
we are able to provide the smoothest experience across all Discord bots.

### Vue.js

We use Vue ecosystem for our frontend website and dashboard. It is very performant and versatile,
allowing the website to load at a rocket speed of under 0.5 seconds for initial content paint.

### CircleCI

Continuous integration is an important part of our workflow. It helps as to find errors quickly and
ensure the quality of the code. CircleCI also provides us with continuous delivery capabilities,
such that every new release will automatically trigger a deployment.

## Self-Hosting

Due to the complex infrastructure, the bot is unfortunately not suitable to for self-hosting at the
moment. Please use our public instance or wait patiently while we are working on a way to run the
bot with Docker.

## Contributing

Want to contribute? Awesome! Please see the [contributing guidelines](CONTRIBUTING.md).

## License

This project is licensed under [GNU Affero General Public License v3.0](LICENSE).
