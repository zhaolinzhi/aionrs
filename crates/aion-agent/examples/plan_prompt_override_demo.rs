//! Demo: verify `plan_mode_override` flows through `build_system_prompt`.
//!
//! Run from the repo root with:
//!   cargo run -p aion-agent --example plan_prompt_override_demo
//!
//! It prints three sections:
//!   1. Plan mode inactive — neither default nor override is injected.
//!   2. Plan mode active, no override — built-in `# Plan Mode` text appears.
//!   3. Plan mode active with override — custom text replaces the default.

use aion_agent::context::{SystemPromptCache, build_system_prompt};

fn divider(label: &str) {
    println!("\n========== {label} ==========");
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| haystack.contains(n))
}

fn main() {
    let cwd = "/tmp/aionrs-plan-demo";
    let model = "claude-sonnet-4-20250514";
    let override_text = "CUSTOM_PLAN_PROMPT_FROM_EXTERNAL_CALLER_42";

    // ---------- Case 1: plan mode OFF ----------
    divider("Case 1: plan_mode_active=false, override=Some(...)");
    {
        let mut cache = SystemPromptCache::new();
        let prompt = build_system_prompt(
            &mut cache,
            None,
            cwd,
            model,
            &[],
            None,
            None,
            false, // plan_mode_active
            Some(override_text),
            false,
        );
        println!("contains '# Plan Mode'? {}", prompt.contains("# Plan Mode"));
        println!(
            "contains override text?  {}",
            prompt.contains(override_text)
        );
        assert!(
            !prompt.contains("# Plan Mode"),
            "plan mode OFF should not inject default"
        );
        assert!(
            !prompt.contains(override_text),
            "plan mode OFF should not inject override either"
        );
        println!("OK");
    }

    // ---------- Case 2: plan mode ON, override=None ----------
    divider("Case 2: plan_mode_active=true, override=None (default text used)");
    {
        let mut cache = SystemPromptCache::new();
        let prompt = build_system_prompt(
            &mut cache,
            None,
            cwd,
            model,
            &[],
            None,
            None,
            true,
            None,
            false,
        );
        println!("contains '# Plan Mode'?     {}", prompt.contains("# Plan Mode"));
        println!(
            "contains 'ExitPlanMode'?   {}",
            prompt.contains("ExitPlanMode")
        );
        println!(
            "contains 'MUST NOT'?       {}",
            prompt.contains("MUST NOT")
        );
        println!(
            "contains override text?    {}",
            prompt.contains(override_text)
        );
        assert!(
            contains_any(&prompt, &["# Plan Mode", "ExitPlanMode", "MUST NOT"]),
            "default plan mode text should appear when override is None"
        );
        println!("OK");
    }

    // ---------- Case 3: plan mode ON, override=Some(...) ----------
    divider("Case 3: plan_mode_active=true, override=Some(...) (default replaced)");
    {
        let mut cache = SystemPromptCache::new();
        let prompt = build_system_prompt(
            &mut cache,
            None,
            cwd,
            model,
            &[],
            None,
            None,
            true,
            Some(override_text),
            false,
        );
        println!(
            "contains override text?  {}",
            prompt.contains(override_text)
        );
        println!(
            "contains '# Plan Mode'?  {}",
            prompt.contains("# Plan Mode")
        );
        println!(
            "contains 'ExitPlanMode'? {}",
            prompt.contains("ExitPlanMode")
        );
        assert!(
            prompt.contains(override_text),
            "override text should be present"
        );
        assert!(
            !prompt.contains("# Plan Mode"),
            "default heading should NOT be present when override is supplied"
        );
        // Print the tail to show where the override lands.
        if let Some(pos) = prompt.find(override_text) {
            println!(
                "override text found at byte {} of {} (last 200 bytes follow)",
                pos,
                prompt.len()
            );
            let tail_start = prompt.len().saturating_sub(200);
            println!("tail: ...{}", &prompt[tail_start..]);
        }
        println!("OK");
    }

    divider("ALL CASES PASSED");
}
