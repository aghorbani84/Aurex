# Plugin development — design constraints, not an API

AUREX does not load or install plugins yet. There is no executable example, ABI,
manifest format or plugin capability grant to depend on. No unrestricted native
plugin loading or scripting backend is included.

A future slice must evaluate sandboxed WebAssembly with bounded memory, fuel/time,
versioned host calls and explicit per-root read/write/network permissions. Default
read-only; destructive actions need normal operation planning and user confirmation.
Untrusted preview content and plugin crashes must be isolated from the application.
Host filesystem and credentials must never be inherited implicitly. Signature/origin
checks, compatibility validation, uninstall/revocation, safe configuration, denial
and resource-exhaustion tests are required before a plugin example is advertised.

Likewise, no AI/MCP filesystem endpoint exists. Future assistant requests need
restricted roots, read-only default, explicit upload/destructive consent and a durable
audit trail. An absent interface is preferable to exposing unrestricted filesystem
access under an "AI-ready" label.
