// SPDX-License-Identifier: MIT OR Apache-2.0
//! The one choice the UFS procedure takes besides the attended runner's own options: whether the UFS host's devfreq clock is pinned low
//! before the ramp (the default, as runs 1 to 9 ran) or left as the image has it (item walleye-ufs-unpinned-ramp-variant: the installed
//! Linux runs unpinned, so the baseline that matters is the unpinned one). Declared once with argenv, as an environment variable: the
//! runner declares and refuses every flag, so a flag of this procedure's own would be refused by it before the procedure ran.
//!
//!   WALLEYE_UFS_PIN=pinned|unpinned procedures/ufs-matched-image-on-the-pixel-2.rs [--dry-run] [--log FILE]

use argenv_resolve::*;

contract! {
    PIN: String = Input {
        key: "pin",
        ty: Type::Enum,
        allowed: &["pinned", "unpinned"],
        env: Some(Env::new("WALLEYE_UFS_PIN")),
        summary: "pinned (the default): the UFS host's devfreq clock is held at 50 MHz first, as runs 1 to 9; unpinned: no governor or set_freq write, the governor, clock and gear are read as found at every ramp step and kept.",
        ..Input::EMPTY
    };
}

/// Whether the UFS host's clock is pinned before the ramp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pin {
    Pinned,
    Unpinned,
}

impl Pin {
    /// The choice from the process's environment; a value that is neither word is an error naming both.
    pub fn from_process_env() -> Result<Pin, String> {
        Pin::from_env(&ProcessEnv)
    }

    /// The choice from `env` (a test gives it a map): none given is `pinned`, as the runs before it.
    pub fn from_env(env: &impl EnvSource) -> Result<Pin, String> {
        // argenv resolves and lints the declaration: a misspelled name or a word outside `allowed` is a refusal, never a silent pin
        let model = Contract::records();
        let resolved = Invocation::from_env(env).resolve(&model);
        let refused: Vec<String> = lint(&model, &resolved)
            .iter()
            .filter(|f| f.severity() == Severity::Error || matches!(f, Finding::UnknownEnv { .. }))
            .map(|f| f.to_string())
            .collect();
        if !refused.is_empty() {
            return Err(format!(
                "{} (WALLEYE_UFS_PIN is pinned or unpinned)",
                refused.join("; ")
            ));
        }
        match Contract::PIN.get_from(&resolved).as_deref().map(str::trim) {
            None | Some("") | Some("pinned") => Ok(Pin::Pinned),
            Some("unpinned") => Ok(Pin::Unpinned),
            Some(other) => Err(format!(
                "WALLEYE_UFS_PIN is {other:?}: it is pinned or unpinned"
            )),
        }
    }

    pub fn is_pinned(self) -> bool {
        self == Pin::Pinned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn env(v: Option<&str>) -> BTreeMap<String, String> {
        v.map(|v| ("WALLEYE_UFS_PIN".to_string(), v.to_string()))
            .into_iter()
            .collect()
    }

    #[test]
    fn the_declaration_holds() {
        assert!(
            Contract::problems().is_empty(),
            "{:#?}",
            Contract::problems()
        );
    }

    #[test]
    fn pinned_is_the_default_and_unpinned_is_asked_for() {
        assert_eq!(Pin::from_env(&env(None)), Ok(Pin::Pinned));
        assert_eq!(Pin::from_env(&env(Some("pinned"))), Ok(Pin::Pinned));
        assert_eq!(Pin::from_env(&env(Some("unpinned"))), Ok(Pin::Unpinned));
        assert_eq!(Pin::from_env(&env(Some(" unpinned\n"))), Ok(Pin::Unpinned));
    }

    #[test]
    fn a_misspelled_name_is_refused_and_never_a_silent_pin() {
        let typo: BTreeMap<String, String> =
            [("WALLEYE_UFS_PN".to_string(), "unpinned".to_string())].into();
        assert!(Pin::from_env(&typo).is_err());
    }

    #[test]
    fn any_other_word_is_refused_and_never_taken_for_pinned() {
        for bad in ["unpin", "no", "0", "UNPINNED", "pinned unpinned"] {
            let e = Pin::from_env(&env(Some(bad))).unwrap_err();
            assert!(e.contains("pinned or unpinned"), "{bad}: {e}");
        }
    }
}
