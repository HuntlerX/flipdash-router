#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    name: "FlipDash Router",
    project_url: "https://flipdash.cash",
    contacts: "https://x.com/HuntlerX",
    policy: "https://github.com/HuntlerX/flipdash-router/blob/main/SECURITY.md",
    source_code: "https://github.com/HuntlerX/flipdash-router"
}
