//! Classify the host-reported default web browser for preview guidance.
//!
//! Hive never queries the operating system itself. The user-setup Skill runs the documented
//! read-only query (Windows `UserChoice` `ProgId`, macOS `LaunchServices` `https` handler, Linux
//! `xdg-settings get default-web-browser`) and records the raw answer; this module only maps it.

/// One recognized browser and whether it is Chromium based.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KnownBrowser {
    pub(crate) name: &'static str,
    pub(crate) chromium: bool,
}

/// Lowercase markers seen in Windows `ProgId` values, macOS bundle IDs and Linux desktop files.
const KNOWN: &[(&[&str], KnownBrowser)] = &[
    (
        &["whale"],
        KnownBrowser {
            name: "Whale",
            chromium: true,
        },
    ),
    (
        &[
            "msedge",
            "microsoft-edge",
            "com.microsoft.edgemac",
            "edgehtm",
        ],
        KnownBrowser {
            name: "Edge",
            chromium: true,
        },
    ),
    (
        &["brave"],
        KnownBrowser {
            name: "Brave",
            chromium: true,
        },
    ),
    (
        &["vivaldi"],
        KnownBrowser {
            name: "Vivaldi",
            chromium: true,
        },
    ),
    (
        &["opera"],
        KnownBrowser {
            name: "Opera",
            chromium: true,
        },
    ),
    (
        &["company.thebrowser", "arc"],
        KnownBrowser {
            name: "Arc",
            chromium: true,
        },
    ),
    (
        &["chromium"],
        KnownBrowser {
            name: "Chromium",
            chromium: true,
        },
    ),
    (
        &["chrome"],
        KnownBrowser {
            name: "Chrome",
            chromium: true,
        },
    ),
    (
        &["firefox", "librewolf"],
        KnownBrowser {
            name: "Firefox",
            chromium: false,
        },
    ),
    (
        &["safari"],
        KnownBrowser {
            name: "Safari",
            chromium: false,
        },
    ),
];

const CHROME: KnownBrowser = KnownBrowser {
    name: "Chrome",
    chromium: true,
};

/// Map one raw default-browser answer to a known browser, or `None` when unrecognized.
pub(crate) fn recognize(raw: &str) -> Option<KnownBrowser> {
    let value = raw.trim().to_ascii_lowercase();
    if value.is_empty() {
        return None;
    }
    KNOWN
        .iter()
        .find(|(markers, _)| markers.iter().any(|marker| value.contains(marker)))
        .map(|(_, browser)| *browser)
}

/// The browser previews should use: the default browser when it is Chromium based, otherwise
/// Chrome. A missing or unrecognized answer also falls back to Chrome.
pub(crate) fn preview_browser(raw: Option<&str>) -> KnownBrowser {
    raw.and_then(recognize)
        .filter(|browser| browser.chromium)
        .unwrap_or(CHROME)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_platform_answers_map_to_chromium_or_fall_back_to_chrome() {
        // Windows ProgIds from HKCU\...\UrlAssociations\https\UserChoice.
        assert_eq!(
            preview_browser(Some("WhaleHTML.6ZRHBO2ZNP55TH")).name,
            "Whale"
        );
        assert_eq!(preview_browser(Some("MSEdgeHTM")).name, "Edge");
        assert_eq!(preview_browser(Some("ChromeHTML")).name, "Chrome");
        assert_eq!(
            preview_browser(Some("FirefoxURL-308046B0AF4A39CB")).name,
            "Chrome"
        );
        // macOS LaunchServices https handler bundle IDs.
        assert_eq!(preview_browser(Some("com.naver.whale")).name, "Whale");
        assert_eq!(
            preview_browser(Some("company.thebrowser.browser")).name,
            "Arc"
        );
        assert_eq!(preview_browser(Some("com.apple.safari")).name, "Chrome");
        // Linux xdg-settings desktop files.
        assert_eq!(preview_browser(Some("brave-browser.desktop")).name, "Brave");
        assert_eq!(preview_browser(Some("chromium.desktop")).name, "Chromium");
        assert_eq!(
            preview_browser(Some("org.mozilla.firefox.desktop")).name,
            "Chrome"
        );
        // Detection failure.
        assert_eq!(preview_browser(None).name, "Chrome");
        assert_eq!(preview_browser(Some("  ")).name, "Chrome");
        assert_eq!(
            preview_browser(Some("unknown-browser.desktop")).name,
            "Chrome"
        );
        assert!(!recognize("firefox").unwrap().chromium);
    }
}
