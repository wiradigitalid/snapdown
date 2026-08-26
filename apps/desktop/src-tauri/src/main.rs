// Desktop Tauri shell - Native entry point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Configure Webview2 runtime memory flags to trim idle browser background RAM usage
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--js-flags=\"--max-old-space-size=128\" --disable-background-networking --disable-features=Translate,OptimizationHints,MediaRouter --renderer-process-limit=2",
    );

    desktop_lib::run();
}
