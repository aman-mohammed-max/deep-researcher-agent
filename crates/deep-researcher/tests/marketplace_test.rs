use deep_researcher_agent::skills::manifest::LoadedSkill;
use deep_researcher_agent::skills::MarketplaceRegistry;

#[tokio::test]
async fn test_marketplace_zero_auth_and_stars_filter() {
    let registry = MarketplaceRegistry::new(10); // Minimum 10 stars

    // Query containing "room" should match local room/reddit crawler skills
    let skills = registry.search_skills("Find flat or room in Kochi").await;
    assert!(!skills.is_empty(), "Expected relevant marketplace skills to be returned");

    for skill in &skills {
        // Enforce strict zero-auth rule
        assert!(
            !skill.requires_auth,
            "Skill '{}' violated zero-auth policy by requiring authentication",
            skill.id
        );
        // Enforce star rating threshold
        assert!(
            skill.stars >= 10,
            "Skill '{}' had {} stars, below min_stars threshold of 10",
            skill.id,
            skill.stars
        );
    }
}

#[tokio::test]
async fn test_marketplace_finance_stock_filter() {
    let registry = MarketplaceRegistry::new(50);
    let skills = registry.search_skills("NVDA stock market price history").await;

    assert!(!skills.is_empty());
    for skill in &skills {
        assert!(!skill.requires_auth);
        assert!(skill.stars >= 50);
    }
}

#[test]
fn test_parse_skill_manifest_valid() {
    let skill_md_content = r#"---
name: "reddit_public_miner"
description: "Zero-auth Reddit search and discussion parser"
requires_auth: false
stars: 450
category: "social_media"
---

# Instructions
Execute public JSON search at https://www.reddit.com/search.json.
Extract upvotes, submission titles, and top comment insights.
"#;

    let parsed = LoadedSkill::parse(skill_md_content).expect("Failed to parse valid SKILL.md");
    assert_eq!(parsed.metadata.name, "reddit_public_miner");
    assert_eq!(parsed.metadata.requires_auth, false);
    assert_eq!(parsed.metadata.stars, 450);
    assert!(parsed.instructions.contains("Execute public JSON search"));
}

#[test]
fn test_parse_skill_manifest_invalid() {
    let invalid_content = "This file does not have frontmatter delimiters.";
    let res = LoadedSkill::parse(invalid_content);
    assert!(res.is_err());
}
