# DeskHelp

A Help Desk bot for DeskThing.

## Setup

1. Acquire this information and put it in a `.env` file:
- Discord Bot Token
- OpenAI API key
- OpenAI API Base URL
- OpenAI Model Name
```sh
DISCORD_TOKEN=
OPENAI_BASE_URL=
OPENAI_API_KEY=
AI_MODEL=
```
2. Run the bot with `cargo run`


## To build multi-arch image and push to GHCR
Assuming you're logged in to GHCR:
`docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/espeon/deskhelp:latest . --push`
