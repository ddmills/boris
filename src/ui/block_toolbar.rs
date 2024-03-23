use bevy::{prelude::*, ui::FocusPolicy};

use crate::{debug::debug_settings::DebugSettings, Block};

use super::Tool;

const BTN_PRESSED: Color = Color::BLUE;
const BTN_NONE: Color = Color::ORANGE_RED;
const BTN_HOVERED: Color = Color::GREEN;
const BTN_TOGGLED: Color = Color::PURPLE;

#[derive(Component)]
pub struct BtnTool {
    tool: Tool,
}

#[derive(Resource)]
pub struct Toolbar {
    pub tool: Tool,
}

pub fn toolbar_select(
    mut toolbar: ResMut<Toolbar>,
    mut btn_query: Query<(&Interaction, &BtnTool, &mut BackgroundColor)>,
) {
    for (interaction, btn, mut bkg) in &mut btn_query {
        match *interaction {
            Interaction::Pressed => {
                toolbar.tool = btn.tool.clone();
                bkg.0 = BTN_PRESSED;
            }
            Interaction::Hovered => {
                bkg.0 = BTN_HOVERED;
            }
            Interaction::None => {
                bkg.0 = BTN_NONE;
            }
        }

        if btn.tool == toolbar.tool {
            bkg.0 = BTN_TOGGLED;
        }
    }
}

pub fn setup_block_toolbar_ui(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                focus_policy: FocusPolicy::Block,
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    border: UiRect::top(Val::Px(2.0)),
                    bottom: Val::Px(0.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                background_color: Color::rgba(0.2, 0.2, 0.2, 0.4).into(),
                border_color: Color::rgb(0.1, 0.1, 0.1).into(),
                ..default()
            },
            Interaction::None,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            ..default()
                        },
                        background_color: BTN_NONE.into(),
                        ..default()
                    },
                    BtnTool { tool: Tool::Mine },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "mine",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            ..default()
                        },
                        background_color: BTN_NONE.into(),
                        ..default()
                    },
                    BtnTool {
                        tool: Tool::BlockInfo,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "info",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            ..default()
                        },
                        background_color: BTN_NONE.into(),
                        ..default()
                    },
                    BtnTool {
                        tool: Tool::TogglePathDebug,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "paths",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            ..default()
                        },
                        background_color: BTN_NONE.into(),
                        ..default()
                    },
                    BtnTool {
                        tool: Tool::ClearBlocks,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "clear",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            ..default()
                        },
                        background_color: BTN_NONE.into(),
                        ..default()
                    },
                    BtnTool {
                        tool: Tool::SpawnColonist,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "colonist",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });

            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            justify_content: JustifyContent::Center,
                            align_content: AlignContent::Center,
                            ..default()
                        },
                        background_color: BTN_NONE.into(),
                        ..default()
                    },
                    BtnTool {
                        tool: Tool::SpawnPickaxe,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "pickaxe",
                        TextStyle {
                            font_size: 18.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });

            vec![
                Block::GRASS,
                Block::DIRT,
                Block::STONE,
                Block::ASHLAR,
                Block::ASHLAR_LARGE,
                Block::LAMP,
                Block::MAGMA,
                Block::LADDER,
            ]
            .into_iter()
            .for_each(|block: Block| {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(48.0),
                                height: Val::Px(48.0),
                                justify_content: JustifyContent::Center,
                                align_content: AlignContent::Center,
                                ..default()
                            },
                            background_color: BTN_NONE.into(),
                            ..default()
                        },
                        BtnTool {
                            tool: Tool::PlaceBlocks(block),
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            block.name(),
                            TextStyle {
                                font_size: 18.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ));
                    });
            });
        });
}
