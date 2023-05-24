# Getting Started:
Hearth can run both the worker and the scheduler in the instance if you do not want to 
setup individual workers and a scheduler.

Start by downloading the binary from Github Releases. And then create a `config.toml` file
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

## Definitions:

## Worker
A worker is anode that handles commands from the client and processes audio. You may have many of these connected to a single scheduler

## Scheduler
A scheduler distributes load across one or many workers using a round-robin algorithm.

## Extra:
See ARCHITECTURE.MD for more details about Hearth's architecture.