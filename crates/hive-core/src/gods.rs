//! Names and display text for the nine functional entry points.
#[derive(Clone, Copy, Debug)]
pub struct God {
    pub name: &'static str,
    pub korean: &'static str,
    pub purpose_en: &'static str,
    pub purpose_ko: &'static str,
    pub role: &'static str,
}

pub const GODS: &[God] = &[
    God {
        name: "athena",
        korean: "아테나",
        purpose_en: "planning",
        purpose_ko: "계획",
        role: "hive-complex-implementer",
    },
    God {
        name: "artemis",
        korean: "아르테미스",
        purpose_en: "research",
        purpose_ko: "조사",
        role: "hive-research-specialist",
    },
    God {
        name: "hephaistos",
        korean: "헤파이스토스",
        purpose_en: "implementation",
        purpose_ko: "구현",
        role: "hive-routine-implementer",
    },
    God {
        name: "apollon",
        korean: "아폴론",
        purpose_en: "writing",
        purpose_ko: "글쓰기",
        role: "hive-article-writer",
    },
    God {
        name: "eileithyia",
        korean: "에일레이티이아",
        purpose_en: "setup",
        purpose_ko: "설정",
        role: "hive-routine-implementer",
    },
    God {
        name: "poseidon",
        korean: "포세이돈",
        purpose_en: "coordination",
        purpose_ko: "조율",
        role: "hive-complex-implementer",
    },
    God {
        name: "demeter",
        korean: "데메테르",
        purpose_en: "knowledge",
        purpose_ko: "지식",
        role: "hive-research-specialist",
    },
    God {
        name: "aither",
        korean: "아이테르",
        purpose_en: "usage controls",
        purpose_ko: "사용량 관리",
        role: "hive-routine-implementer",
    },
    God {
        name: "hades",
        korean: "하데스",
        purpose_en: "cleanup",
        purpose_ko: "정리",
        role: "hive-routine-implementer",
    },
];
impl God {
    #[must_use]
    pub fn display_name(&self) -> String {
        let mut name = self.name.to_owned();
        name[0..1].make_ascii_uppercase();
        name
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
    let lower = name.to_lowercase();
    let canonical = match lower.as_str() {
        "minerva" | "미네르바" | "아테나" => "athena",
        "아르테미스" => "artemis",
        "hephaestus" | "헤파이스토스" => "hephaistos",
        "apollo" | "아폴로" | "아폴론" => "apollon",
        "eleuthia" | "에일레이티이아" | "에일레이티아" | "엘레이티아" | "엘레우티아" => {
            "eileithyia"
        }
        "포세이돈" => "poseidon",
        "데메테르" => "demeter",
        "aether" | "에테르" | "아이테르" => "aither",
        "하데스" => "hades",
        other => other,
    };
    GODS.iter().find(|god| god.name == canonical)
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
        }
        assert!(find("unknown").is_none());
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
