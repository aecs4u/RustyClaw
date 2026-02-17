# Configuration

RustyClaw reads configuration from:

- `~/.rustyclaw/config.toml`

You can create or update it with:

```bash
rustyclaw configure
```

Minimal example:

```toml
settings_dir = "~/.rustyclaw"

[model]
provider = "openrouter"
model = "gpt-4.1"
base_url = "https://openrouter.ai/api/v1"

[sandbox]
mode = "auto"

[context_compaction]
enabled = true
strategy = "hybrid"

[rate_limit]
enabled = true
capacity = 30.0
refill_per_sec = 5.0
control_cost = 2.0

[mdns]
enabled = false
mode = "minimal"

[webhook_triggers]
enabled = false
listen = "127.0.0.1:8787"
path_prefix = "/webhook"
```

Reference examples:

- `config.example.toml`
- `docs/SANDBOX.md`
- `docs/SECURITY.md`
