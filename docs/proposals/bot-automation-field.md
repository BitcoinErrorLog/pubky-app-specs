# Proposal: optional `automation` block on `profile.json`

**Status:** implemented on `proposal/bot-automation-field` (BitcoinErrorLog fork). Not version-bumped; maintainers choose the crate/npm cut.

**Implied semver:** this crate is `0.6.x`. The JSON change is additive and backward compatible, but `PubkyAppUser::new` / WASM `createUser` gain a parameter and a new exported type. Under Cargo's 0.x rules that is a **minor bump (`0.7.0`)**. After 1.0 it would be major. Do not treat it as a patch: native and JS callers of the constructor must be updated (the extra argument may be omitted in JS and is `None` / `null` for human profiles).

## Motivation

Today the spec has no bot notion. A profile is a person-shaped object: name, bio, image, links, status. That is already insufficient for accounts that are clearly machines.

[Jeb](https://pubky.app) is a live Synonym bot at pubky `9o6xrx8wgqu48dmb47uep6w3dgbwdnf5jgw83gbeuxg9yi7x444y`. It posts, it has a public policy article, and it is operated by people who already have their own pubkys. Clients have no structured way to:

- mark the account as automated,
- name the operator,
- list what the bot is allowed to do,
- link source and policy.

Without a field, apps invent ad-hoc bio text or dump operator URLs into `links`. Indexers cannot query "bots operated by X". Users cannot tell a scripted account from a human one except by rumor.

This proposal adds an optional `automation` object on the existing user profile. Attribution of posts and tags remains the bot's own key; the block only declares *who runs it* and *under what terms*.

## Schema

On `PubkyAppUser` / `/pub/pubky.app/profile.json`:

```json
{
  "name": "Jeb",
  "bio": "Synonym bot",
  "automation": {
    "operator": "operrr8wsbpr3ue9d4qj41ge1kcc6r7fdiy6o3ugjrrhi4y77rdo",
    "capabilities": ["post", "tag", "follow"],
    "source": "https://github.com/synonymdev/jeb",
    "policy": "pubky://9o6xrx8wgqu48dmb47uep6w3dgbwdnf5jgw83gbeuxg9yi7x444y/pub/pubky.app/posts/<id>"
  }
}
```

| Field | Type | Required when block present | Meaning |
| --- | --- | --- | --- |
| `operator` | Pubky id (z32, 52 chars) | yes | Human or organization pubky that operates this bot. |
| `capabilities` | `string[]` | yes (array may be empty) | What this bot is declared to do. |
| `source` | URL | yes | Repository, package, or equivalent source. |
| `policy` | URL | yes | Public operating policy (article, post, or https page). |

The profile remains valid with `automation` omitted. `null` is treated as omitted (`#[serde(default, skip_serializing_if = "Option::is_none")]`). Partial objects (e.g. missing `operator`) are rejected.

## Validation

Follows the crate's existing style: sanitize trims / URL-normalizes; validate rejects. Limits live in `VALIDATION_LIMITS`.

| Rule | Limit / check |
| --- | --- |
| Operator | Non-empty; `PubkyId::try_from` (same encoding rules as other pubkys). |
| Capabilities count | ≤ 16 (`user_automation_capabilities_max_count`) |
| Capability length | 1–40 Unicode scalars (`user_automation_capability_max_length`) |
| Capability shape | Lowercase kebab-case: `[a-z0-9]+(?:-[a-z0-9]+)*`. Not rewritten to lowercase. |
| Source / policy | Non-empty; `url::Url::parse`; ≤ 300 chars (`user_automation_url_max_length`) |

Capability tokens are a declaration, not an ACL. Clients must not treat them as cryptographic authorization.

## Rendering guidance (Pubky App)

When `automation` is present:

1. **Badge** — a persistent "Bot" (or equivalent) mark on the profile header and on posts authored by this pubky, so the account is not mistaken for a human.
2. **Operator link** — render `operator` as a profile link ("Operated by …"), using the operator's `profile.json` when available.
3. **What this bot can do** — list `capabilities` as human-readable chips (map known tokens like `post`, `tag`, `follow`; show unknown kebab-case tokens as-is).
4. **Policy link** — a control that opens `policy`. Prefer this over burying the policy in bio.
5. **Source link** — secondary; useful for developers, not required in the primary chrome.

Human profiles without the block must look unchanged.

## Why not a new object

A separate `/pub/pubky.app/bot.json` (or similar) would split identity: clients already load `profile.json` for the header. A second fetch, second URI, and second resource type would delay the badge and invite profiles that forget the companion file. Bots are still users; they post as users. An additive optional field keeps one object, one path, and old clients that ignore unknown JSON keys.

## Why not reserved tags

Reserved tag conventions for machine output (`#bot`, `#jeb`, etc.) would pollute the tag graph, collide with user language, and fail as soon as someone tags a human post with the reserved label. Attribution is already the signing key. The operator/capability/policy data is profile metadata, not content classification.

## Backward compatibility

- Existing profiles without `automation` continue to validate.
- Indexers and apps that ignore unknown fields keep working.
- Apps that `deny_unknown_fields` (this crate does not) would need a bump before they can ingest bot profiles.
- Rust/WASM constructor signatures change (sixth argument / extra `createUser` parameter). JS callers that omit the last argument get `undefined` → no block.

## Open questions

1. **Should Nexus index `automation.operator`?** A reverse index ("bots operated by X") would make operator pages and moderation useful. This proposal does not require it; it is the main indexer follow-up.
2. Should `capabilities` be a closed enum later, or stay open kebab-case forever?
3. May `operator` be a homeserver or organization key rather than a personal profile?
4. Should a bot that omits `automation` still be badgeable by a client-side allowlist (e.g. known Jeb pubky), or only via this field?
