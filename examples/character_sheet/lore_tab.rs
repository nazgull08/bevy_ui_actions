use bevy::prelude::*;
use bevy_ui_actions::prelude::*;



pub fn setup_lore_registry(commands: &mut Commands) {
    let mut registry = TopicRegistry::default();

    registry.insert(
        "iron_helm",
        TopicEntry::new(
            "Iron Helm",
            "Standard-issue helm forged in the [Ironvale|ironvale] foundries. \
             Provides decent protection at the cost of peripheral vision. \
             The [Royal Guard|royal_guard] swears by them.",
        )
        .with_category("equipment"),
    );

    registry.insert(
        "ironvale",
        TopicEntry::new(
            "Ironvale",
            "A mining town nestled in the northern mountains, renowned for its \
             [iron|iron_ore] deposits. The [Ironvale foundries|foundries] supply \
             arms to much of the kingdom.",
        )
        .with_category("locations"),
    );

    registry.insert(
        "foundries",
        TopicEntry::new(
            "Ironvale Foundries",
            "Massive forges carved into the mountainside, powered by underground \
             [lava channels|lava]. Master smiths here craft everything from \
             common blades to enchanted [arcane equipment|arcane_craft].",
        )
        .with_category("locations"),
    );

    registry.insert(
        "iron_ore",
        TopicEntry::new(
            "Iron Ore",
            "The lifeblood of [Ironvale|ironvale]. Rich veins run deep beneath \
             the mountains. Miners report strange [whispers|deep_whispers] from \
             the deepest shafts.",
        )
        .with_category("materials"),
    );

    registry.insert(
        "lava",
        TopicEntry::new(
            "Lava Channels",
            "Natural magma flows harnessed by the [foundries|foundries] for \
             smelting. The channels were first mapped by the \
             [Dwarven Cartographers|dwarves], long before humans settled the region.",
        )
        .with_category("locations"),
    );

    registry.insert(
        "dwarves",
        TopicEntry::new(
            "Dwarven Cartographers",
            "An ancient order that mapped the underground networks beneath \
             [Ironvale|ironvale]. Their stone tablets describe a \
             [sealed chamber|sealed_chamber] at the mountain's heart.",
        )
        .with_category("factions"),
    );

    registry.insert(
        "sealed_chamber",
        TopicEntry::new(
            "The Sealed Chamber",
            "A vault deep below [Ironvale|ironvale], sealed by the \
             [dwarves|dwarves] with runes no living smith can read. \
             The [deep whispers|deep_whispers] seem to emanate from behind its door.",
        )
        .with_category("locations"),
    );

    registry.insert(
        "deep_whispers",
        TopicEntry::new(
            "Deep Whispers",
            "Miners in the lowest shafts of [Ironvale|ironvale] report hearing \
             faint voices. The words are never clear. Those who listen too long \
             develop an obsession with the [sealed chamber|sealed_chamber]. \
             The [Royal Guard|royal_guard] has forbidden further excavation.",
        )
        .with_category("mysteries"),
    );

    registry.insert(
        "royal_guard",
        TopicEntry::new(
            "The Royal Guard",
            "Elite soldiers equipped with [Ironvale|ironvale]-forged arms. \
             They enforce the king's law and recently posted a garrison near \
             the mines to prevent anyone from reaching the \
             [sealed chamber|sealed_chamber].",
        )
        .with_category("factions"),
    );

    registry.insert(
        "arcane_craft",
        TopicEntry::new(
            "Arcane Crafting",
            "A rare art combining smithing with magical infusion. The \
             [foundries|foundries] employ a handful of mage-smiths who channel \
             energy into metal during forging. Resulting weapons shimmer with \
             an inner light. The practice is controversial — the \
             [Royal Guard|royal_guard] considers it borderline heresy.",
        )
        .with_category("knowledge"),
    );

    commands.insert_resource(registry);
}

pub fn spawn_lore_tab(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            min_height: Val::Px(0.0),
            row_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|col| {
            col.ui_text(TextRole::Heading, "Journal");

            let ht_config = HyperTextConfig {
                text_role: TextRole::Body,
                link_color: Color::srgb(0.4, 0.65, 0.95),
                link_hover_color: Color::srgb(0.6, 0.8, 1.0),
                visited_link_color: Some(Color::srgb(0.55, 0.4, 0.75)),
                width: Val::Percent(100.0),
                font_size: Some(18.0),
            };
            let ht_config2 = ht_config.clone();

            col.spawn_scroll_view_with(
                ScrollViewConfig {
                    height: Val::Percent(100.0),
                    background: Some(Color::srgb(0.08, 0.08, 0.10)),
                    show_scrollbar: true,
                    scrollbar_track: Color::srgba(0.12, 0.12, 0.15, 0.6),
                    scrollbar_thumb: Color::srgba(0.4, 0.4, 0.45, 0.7),
                    ..default()
                },
                move |scroll| {
                    let config = &ht_config2;

                    scroll
                        .spawn((
                            TopicContainer {
                                hypertext_config: config.clone(),
                                header_role: TextRole::Label,
                            },
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(16.0),
                                padding: UiRect::all(Val::Px(16.0)),
                                width: Val::Percent(100.0),
                                ..default()
                            },
                        ))
                        .with_children(|content| {
                            content.ui_text(TextRole::Label, "Day 47 — Expedition Notes");

                            content.spawn_hypertext(
                                config,
                                "Arrived at [Ironvale|ironvale] after three days on the mountain road. \
                                 The town is smaller than I expected — a handful of stone buildings \
                                 huddled around the entrance to the [foundries|foundries]. \
                                 Smoke rises day and night.",
                            );

                            content.spawn_hypertext(
                                config,
                                "The locals speak of [strange sounds|deep_whispers] from the mines. \
                                 A patrol of the [Royal Guard|royal_guard] turned me away from \
                                 the lower shafts. They offered no explanation.",
                            );

                            content.spawn_hypertext(
                                config,
                                "I traded for an [Iron Helm|iron_helm] at the market. \
                                 Good craftsmanship — the [Ironvale|ironvale] smiths know their trade. \
                                 The merchant mentioned something about [arcane forging|arcane_craft] \
                                 but clammed up when a guardsman walked past.",
                            );

                            content.ui_text(TextRole::Label, "Day 51 — Discovery");

                            content.spawn_hypertext(
                                config,
                                "Found a reference to the [sealed chamber|sealed_chamber] in an old \
                                 [dwarven|dwarves] manuscript at the tavern. The innkeeper says it \
                                 was left behind by a scholar who vanished weeks ago. \
                                 The text describes [iron ore|iron_ore] deposits of unusual purity \
                                 near the sealed door.",
                            );

                            content.spawn_hypertext(
                                config,
                                "I must find a way past the [guards|royal_guard]. Whatever is behind \
                                 that door, someone doesn't want it found.",
                            );

                            content.ui_text_styled(
                                "\u{2139} Click highlighted words to expand topics \u{2022} Scroll to read more",
                                13.0,
                                Color::srgb(0.4, 0.4, 0.48),
                            );
                        });
                },
            );
        });
}
