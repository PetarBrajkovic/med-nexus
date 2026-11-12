<!-- Resolved in https://github.com/PetarBrajkovic/med-nexus/issues/14 -->
# Server-side sessions in a cookie, not JWT

The Rust API authenticates browsers with **server-side sessions** (tower-sessions, MySQL store). The browser holds only an opaque session ID in an `HttpOnly`, `SameSite=Lax` cookie. The session stores only the `user_id`; the role is read from MySQL on every request. We chose this because the frontend is served same-origin by Rust (ADR 0002), so cookies come for free, and because a session can be revoked: logout, demotion and expiry take effect on the next request. JavaScript never sees a credential.

## Considered options

- **JWT in `localStorage`, sent as `Authorization: Bearer`.** Rejected: any XSS can read and exfiltrate the token, and logout or demotion can't take effect before expiry without a server-side blocklist, which brings back server state anyway.
- **JWT in an `HttpOnly` cookie.** Rejected: it fixes the XSS exposure but keeps the revocation problem, and it adds signing/expiry/refresh concepts on top of cookie handling.

## Consequences

- Cookies are sent automatically, so CSRF has to be handled: `SameSite=Lax`, plus an **Origin check** on every POST/PUT/DELETE (this covers multipart PDF upload), plus JSON-only bodies elsewhere.
- One MySQL lookup per request for the session and one for the role. That's fine at this scale.
- The session ID is rotated on login (prevents session fixation), and logout deletes the session row. Sessions expire after 8 h of inactivity, and every request extends them.
- There are no API tokens: a non-browser client that calls Rust must log in and keep the cookie.
