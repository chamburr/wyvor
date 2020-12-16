# Contributing Guide

Hi, thanks for your interest in contributing to Wyvor! We'd love your help to make Wyvor even better
than it is today. As a contributor, please be sure follow our set of guidelines below.

-   [Issues and Bugs](#issues-and-bugs)
-   [Pull Requests](#pull-requests)
-   [Commit Convention](#commit-convention)
-   [Project Structure](#project-structure)
-   [Questions](#questions)
-   [Code of Conduct](#code-of-conduct)

## Issues and Bugs

We track bugs and features using the GitHub issue tracker. If you come across any bugs or have
feature suggestions, please let us know by submitting an issue, or even better, making a pull
request.

## Pull Requests

Please follow these guidelines related to submitting a pull request.

-   The `master` branch is a snapshot of the latest stable release. Please do not make pull requests
    against the `master` branch, use the `dev` branch instead.
-   Always format your code with `scripts/format.sh` and make sure `scripts/lint.sh` returns no
    warnings or errors before opening a pull request.
-   Follow our commit conventions below. For subsequent commits to the pull request it is okay not
    to follow them, because they will be eventually squashed.

## Commit Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org) to allow for more readable
messages in the commit history. More importantly, they are also used for generating the changelog.

The commit message must follow this format:

```
<type>(<scope>): <description>
<BLANK LINE>
<body>
<BLANK LINE>
<footer>
```

Additionally, the maximum length of each line must not exceed 72 characters.

### Header

The header is mandatory.

The type must be one of the following, scope is optional and can be decided at your discretion.

-   `build`: Changes to the build system or dependencies.
-   `ci`: Changes to our CI configuration files and scripts.
-   `chore`: Miscellaneous change.
-   `docs`: Changes only the documentation.
-   `feat`: Implements a new feature.
-   `fix`: Fixes an existing bug.
-   `perf`: Improves the performance of the code.
-   `refactor`: Changes to code neither fixes a bug nor adds a feature.
-   `style`: Changes to code that do not affect its functionality.
-   `test`: Adding missing tests or correcting existing tests.

### Body

The body is optional and can contain additional information, such as motivation, for the commit.

### Footer

The footer is optional and should contain any information about breaking changes. It is also the
place to reference GitHub issues that the commit closes.

Breaking changes should start with `BREAKING CHANGE:` with a space or two newlines. The rest of the
commit message is then used for explaining the change.

## Project Structure

These are the various components of Wyvor.

-   `api`: Handles HTTP API requests and connects to Andesite.
    -   `migrations`: Database migration scripts for Diesel.
    -   `src`: Contains the source code.
        -   `db`: Handles PostgreSQL and Redis databases.
        -   `models`: Structs for the database schema.
        -   `routes`: Routes and handlers for the HTTP API.
        -   `utils`: Miscellaneous utility functions.
-   `bot`: Handles messages from RabbitMQ queue and commands.
    -   `commands`: Handles the bot commands.
    -   `common`: Common variables for easy access.
    -   `config`: Bot configurations.
    -   `core`: Core part of the bot, such as event handlers.
    -   `utils`: Miscellaneous utility functions.
-   `examples`: Example configuration files.
-   `scripts`: Utility bash scripts.
-   `web`: The frontend website and dashboard components.
    -   `assets`: Un-compiled assets such as styles and images.
    -   `components`: Reusable components for various pages.
    -   `content`: Markdown contents for various pages.
    -   `layout`: Core application feel and layout.
    -   `middleware`: Functions ran before rendering a page.
    -   `pages`: The application's views and routes.
    -   `plugins`: Miscellaneous utility plugins.
    -   `static`: Static files at the root of website.
    -   `store`: Manages state across the application.

## Questions

Have a question? Please avoid opening issues for general questions. Instead, it is much better to
ask your question on our [Discord server](https://wyvor.xyz/support).

## Code of Conduct

This project is governed by [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md).
