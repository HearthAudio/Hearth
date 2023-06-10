# Getting Started:
Hearth can run both the worker and the scheduler in the instance if you do not want to
setup individual workers and a scheduler.

Start by downloading the binary from Github Releases https://github.com/Hearth-Industries/Hearth/releases. And then create a `config.toml` file
in that directory and configure like this:
```toml
[roles]
# If you want to run the worker and scheduler 
# in the same instance you can setup roles here
worker = true
scheduler = true

[config]
# Set discord bot id and token
discord_bot_id = 1928467321045049352
discord_bot_token = "TOKEN-HERE"

[kafka]
# Configure Hearth so that it can connect to your Kafka Instance
kafka_topic = "communication"
kafka_uri = "kafka-uri"
kafka_use_ssl = true
kafka_ssl_cert = "service.cert"
kafka_ssl_key = "service.key"
kafka_ssl_ca = "ca.pem"
```
If your Kafka Instance has SSL then you can configure that inside of the config.
If you want to use SASL authentication instead of SSL you can include the following in you config instead of the SSL config:
```toml
kafka_use_sasl = true
kafka_username = "USERNAME_HERE"
kafka_password = "PASSWORD_HERE"
```


## Configuration:
### Sentry
If you want you can configure Sentry to report errors by adding a sentry_url under the `[config]` header:
```toml
sentry_url = "SENTRY-URL"
```
This will report any errors back to Sentry.
### Log Levels
If you want to adjust the log level (The defualt is `INFO`) you can set the log level under the `[config]` header like this:
```toml
log_level = "DEBUG"
```

## Running Multiple Workers
Running multiple workers is super easy with Hearth! All you have todo is copy the config from your scheduler and your SSL config for Kafka. Change `scheduler = true` to `scheduler = false` so your worker is only a worker and not a scheduler. Then just restart your scheduler. And your done!

## Definitions:

## Worker
A worker is anode that handles commands from the client and processes audio. You may have many of these connected to a single scheduler

## Scheduler
A scheduler distributes load across one or many workers using a round-robin algorithm.

## Extra:
See ARCHITECTURE.MD for more details about Hearth's architecture.

## A Note On Optimization:
When using Hearth and Lavalink for that matter. You should always use Ogg Vorbis files (.ogg) because if you do not use this format Lavalink & Hearth have to downscale it to ogg vorbis in realtime. As it is the only audio format Discord supports. So it is better to downscale it ahead of time then at runtime. This will make hearth vastly better than Lavalink as it is no longer CPU bound when used like this.
