//! Names and display text for the nine functional entry points.
#[derive(Clone, Copy, Debug)]
pub struct God {
    pub name: &'static str,
    pub korean: &'static str,
    pub purpose_en: &'static str,
    pub purpose_ko: &'static str,
    pub role: &'static str,
    pub display_name: &'static str,
    pub description_en: &'static str,
    pub description_ko: &'static str,
    pub aliases: &'static [&'static str],
    pub commands: &'static [&'static str],
}

include!("gods_generated.rs");

impl God {
    #[must_use]
    pub fn display_name(&self) -> String {
        self.display_name.to_owned()
    }
    #[must_use]
    pub fn prefix(&self) -> String {
        format!("[{}]", self.display_name())
    }
    #[must_use]
    pub fn announcement(&self, korean: bool) -> String {
        if korean {
            format!("{} 호출 — {}", self.korean, self.purpose_ko)
        } else {
            format!("Calling upon {} — {}", self.display_name(), self.purpose_en)
        }
    }
}
#[must_use]
pub fn find(name: &str) -> Option<&'static God> {
    GODS.iter().find(|god| {
        god.name.eq_ignore_ascii_case(name)
            || god.korean == name
            || god
                .aliases
                .iter()
                .any(|alias| alias.eq_ignore_ascii_case(name))
    })
}

#[must_use]
pub fn command_owner(command: &str) -> Option<&'static str> {
    GODS.iter()
        .find(|god| god.commands.contains(&command))
        .map(|god| god.name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gods_aliases_and_korean_resolve_to_canonical_names() {
        for (alias, canonical) in [
            ("Minerva", "athena"),
            ("Hephaestus", "hephaistos"),
            ("Apollo", "apollon"),
            ("Eleuthia", "eileithyia"),
            ("Aether", "aither"),
        ] {
            assert_eq!(find(alias).unwrap().name, canonical);
        }
        for god in GODS {
            assert_eq!(find(god.korean).unwrap().name, god.name);
            assert_eq!(find(&god.name.to_uppercase()).unwrap().name, god.name);
            for alias in god.aliases {
                assert_eq!(find(alias).unwrap().name, god.name);
            }
        }
        assert!(find("unknown").is_none());
    }
    #[test]
    fn gods_commands_have_one_owner_from_the_same_table() {
        let mut commands = std::collections::BTreeSet::new();
        for god in GODS {
            for command in god.commands {
                assert!(commands.insert(command), "duplicate command: {command}");
                assert_eq!(command_owner(command), Some(god.name));
            }
        }
        assert_eq!(command_owner("summon"), Some("poseidon"));
        assert_eq!(command_owner("unknown"), None);
    }
    #[test]
    fn gods_nine_by_two_language_display_snapshots() {
        let expected_en = [
            "Calling upon Athena — planning",
            "Calling upon Artemis — research",
            "Calling upon Hephaistos — implementation",
            "Calling upon Apollon — writing",
            "Calling upon Eileithyia — setup",
            "Calling upon Poseidon — coordination",
            "Calling upon Demeter — knowledge",
            "Calling upon Aither — usage controls",
            "Calling upon Hades — cleanup",
        ];
        let expected_ko = [
            "아테나 호출 — 계획",
            "아르테미스 호출 — 조사",
            "헤파이스토스 호출 — 구현",
            "아폴론 호출 — 글쓰기",
            "에일레이티이아 호출 — 설정",
            "포세이돈 호출 — 조율",
            "데메테르 호출 — 지식",
            "아이테르 호출 — 사용량 관리",
            "하데스 호출 — 정리",
        ];
        for (i, god) in GODS.iter().enumerate() {
            assert_eq!(god.announcement(false), expected_en[i]);
            assert_eq!(god.announcement(true), expected_ko[i]);
            assert_eq!(god.prefix(), format!("[{}]", god.display_name()));
        }
    }
}
