use bevy::{prelude::*, ui::FocusPolicy, utils::HashMap};

use crate::{items::image_loader_settings, structures::BlueprintType, BlockType};

use super::Tool;

const BTN_PRESSED: Color = Color::rgb(39. / 255., 55. / 255., 66. / 255.);
const BTN_NONE: Color = Color::rgb(55. / 255., 79. / 255., 94. / 255.);
const BTN_HOVERED: Color = Color::rgb(42. / 255., 53. / 255., 71. / 255.);
const BTN_TOGGLED: Color = Color::rgb(78. / 255., 44. / 255., 44. / 255.);

#[derive(Component)]
pub struct BtnTool {
    tool: Tool,
}

#[derive(Component)]
pub struct BtnSubmenu {
    submenu: SubmenuType,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubmenuType {
    Blocks,
    Structures,
}

#[derive(Resource)]
pub struct Toolbar {
    pub tool: Tool,
    pub submenu: Option<SubmenuType>,
    pub submenus: HashMap<SubmenuType, Entity>,
}

pub fn on_toolbar_submenu_btn(
    mut cmd: Commands,
    mut toolbar: ResMut<Toolbar>,
    mut btn_query: Query<(&Interaction, &BtnSubmenu, &mut BackgroundColor), Changed<Interaction>>,
) {
    let current_submenu = toolbar.submenu;

    for (interaction, btn, mut bkg) in &mut btn_query {
        match *interaction {
            Interaction::Pressed => {
                println!("pressed");

                if Some(btn.submenu) == current_submenu {
                    toolbar.submenu = None;
                } else {
                    toolbar.submenu = Some(btn.submenu);
                }

                bkg.0 = BTN_PRESSED;
            }
            Interaction::Hovered => {
                bkg.0 = BTN_HOVERED;
            }
            Interaction::None => {
                bkg.0 = BTN_NONE;
            }
        }

        if Some(btn.submenu) == toolbar.submenu {
            bkg.0 = BTN_TOGGLED;
        }
    }

    if current_submenu.is_some() && current_submenu != toolbar.submenu {
        if let Some(entity) = toolbar.submenus.get(&current_submenu.unwrap()) {
            cmd.entity(*entity).insert(Visibility::Hidden);
        };
    }

    if let Some(submenu_type) = toolbar.submenu {
        if let Some(entity) = toolbar.submenus.get(&submenu_type) {
            cmd.entity(*entity).insert(Visibility::Visible);
        };
    }
}

pub fn on_toolbar_tool_btn(
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

fn tool_btn(
    parent: &mut ChildBuilder,
    name: impl Into<String>,
    tool: Tool,
    icon: Option<Handle<Image>>,
    font: Handle<Font>,
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(16.), Val::Px(8.)),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.),
                    display: Display::Flex,
                    ..default()
                },
                background_color: BTN_NONE.into(),
                ..default()
            },
            BtnTool { tool },
        ))
        .with_children(|p1| {
            if let Some(img) = icon {
                p1.spawn(ImageBundle {
                    image: UiImage::new(img),
                    style: Style {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        // margin: UiRect::right(Val::Px(8.)),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            p1.spawn(TextBundle {
                text: Text::from_section(
                    name,
                    TextStyle {
                        font_size: 18.0,
                        font,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                ),
                ..Default::default()
            });
        });
}

fn toggle_submenu_btn(
    parent: &mut ChildBuilder,
    name: impl Into<String>,
    icon: Option<Handle<Image>>,
    font: Handle<Font>,
    submenu: SubmenuType,
) {
    parent
        .spawn((
            ButtonBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(16.), Val::Px(8.)),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.),
                    display: Display::Flex,
                    ..default()
                },
                background_color: BTN_NONE.into(),
                ..default()
            },
            BtnSubmenu { submenu },
        ))
        .with_children(|p1| {
            if let Some(img) = icon {
                p1.spawn(ImageBundle {
                    image: UiImage::new(img),
                    style: Style {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        // margin: UiRect::right(Val::Px(8.)),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            }
            p1.spawn(TextBundle::from_section(
                name,
                TextStyle {
                    font_size: 18.0,
                    font,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });
}

fn tool_group(
    parent: &mut ChildBuilder,
    name: impl Into<String>,
    font: Handle<Font>,
    spawn_children: impl FnOnce(&mut ChildBuilder),
) {
    parent
        .spawn((
            NodeBundle {
                focus_policy: FocusPolicy::Block,
                style: Style {
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(8.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            Interaction::None,
        ))
        .with_children(|p2| {
            p2.spawn((
                NodeBundle {
                    focus_policy: FocusPolicy::Block,
                    style: Style {
                        padding: UiRect::axes(Val::Px(16.), Val::Px(4.)),
                        ..default()
                    },
                    ..default()
                },
                Interaction::None,
            ))
            .with_children(|p3| {
                p3.spawn(TextBundle {
                    text: Text::from_section(
                        name,
                        TextStyle {
                            font_size: 18.0,
                            font,
                            color: Color::rgb(0.7, 0.7, 0.7),
                        },
                    ),
                    ..Default::default()
                });
            });

            p2.spawn((
                NodeBundle {
                    focus_policy: FocusPolicy::Block,
                    style: Style {
                        border: UiRect::all(Val::Px(2.0)),
                        padding: UiRect::all(Val::Px(8.0)),
                        column_gap: Val::Px(8.0),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    background_color: Color::rgba(0.2, 0.2, 0.2, 0.4).into(),
                    border_color: Color::rgb(0.1, 0.1, 0.1).into(),
                    ..default()
                },
                Interaction::None,
            ))
            .with_children(spawn_children);
        });
}

pub fn setup_block_toolbar_ui(
    mut cmd: Commands,
    mut toolbar: ResMut<Toolbar>,
    asset_server: Res<AssetServer>,
) {
    // let fnt1 = asset_server.load("fonts/Averia_Serif/AveriaSerifLibre-Regular.ttf");
    // let fnt2 = asset_server.load("fonts/Yantramanav/Yantramanav-Black.ttf");
    // let fnt2 = asset_server.load("fonts/Averia_Serif/AveriaSerifLibre-Regular.ttf");
    let fnt2 = asset_server.load("fonts/Yantramanav/Yantramanav-Black.ttf");
    let fnt1 = asset_server.load("fonts/Yantramanav/Yantramanav-Black.ttf");
    let icon_pickaxe =
        asset_server.load_with_settings("textures/icon_pickaxe.png", image_loader_settings);
    let icon_axe = asset_server.load_with_settings("textures/icon_axe.png", image_loader_settings);
    let icon_hammer =
        asset_server.load_with_settings("textures/icon_hammer.png", image_loader_settings);

    cmd.spawn((
        NodeBundle {
            focus_policy: FocusPolicy::Block,
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                bottom: Val::Px(0.0),
                padding: UiRect::all(Val::Px(8.0)),
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        Interaction::None,
    ))
    .with_children(|p1| {
        tool_group(p1, "ORDERS", fnt2.clone(), |p2| {
            tool_btn(p2, "Mine", Tool::Mine, Some(icon_pickaxe), fnt1.clone());
            tool_btn(p2, "Chop", Tool::Chop, Some(icon_axe), fnt1.clone());
        });
    })
    .with_children(|p1| {
        tool_group(p1, "BUILD", fnt2.clone(), |p2| {
            toggle_submenu_btn(
                p2,
                "Build",
                Some(icon_hammer),
                fnt1.clone(),
                SubmenuType::Structures,
            );
        });
    });

    cmd.spawn((
        NodeBundle {
            focus_policy: FocusPolicy::Block,
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: Val::Px(0.0),
                padding: UiRect::all(Val::Px(8.0)),
                align_content: AlignContent::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        Interaction::None,
    ))
    .with_children(|p1| {
        tool_group(p1, "DEBUG", fnt2.clone(), |p2| {
            tool_btn(p2, "Path", Tool::TogglePathDebug, None, fnt1.clone());
            tool_btn(p2, "Info", Tool::BlockInfo, None, fnt1.clone());
        });
    })
    .with_children(|p1| {
        tool_group(p1, "SPAWN", fnt2.clone(), |p2| {
            tool_btn(p2, "Colonist", Tool::SpawnColonist, None, fnt1.clone());
            tool_btn(p2, "Axe", Tool::SpawnAxe, None, fnt1.clone());
            tool_btn(p2, "Pickaxe", Tool::SpawnPickaxe, None, fnt1.clone());
            toggle_submenu_btn(p2, "Block", None, fnt1.clone(), SubmenuType::Blocks);
        });
    });

    let block_submenu = cmd
        .spawn((
            NodeBundle {
                focus_policy: FocusPolicy::Block,
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    top: Val::Px(80.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            Interaction::None,
        ))
        .with_children(|p1| {
            tool_group(p1, "BLOCKS", fnt2.clone(), |p2| {
                tool_btn(p2, "Clear", Tool::ClearBlocks, None, fnt1.clone());
                tool_btn(
                    p2,
                    "Stone",
                    Tool::PlaceBlocks(BlockType::STONE),
                    None,
                    fnt1.clone(),
                );
                tool_btn(
                    p2,
                    "Grass",
                    Tool::PlaceBlocks(BlockType::GRASS),
                    None,
                    fnt1.clone(),
                );
                tool_btn(
                    p2,
                    "Dirt",
                    Tool::PlaceBlocks(BlockType::DIRT),
                    None,
                    fnt1.clone(),
                );
                tool_btn(
                    p2,
                    "Ashlar",
                    Tool::PlaceBlocks(BlockType::ASHLAR),
                    None,
                    fnt1.clone(),
                );
                tool_btn(
                    p2,
                    "Smooth",
                    Tool::PlaceBlocks(BlockType::ASHLAR_LARGE),
                    None,
                    fnt1.clone(),
                );
                tool_btn(
                    p2,
                    "Magma",
                    Tool::PlaceBlocks(BlockType::MAGMA),
                    None,
                    fnt1.clone(),
                );
            });
        })
        .id();

    let build_submenu = cmd
        .spawn((
            NodeBundle {
                focus_policy: FocusPolicy::Block,
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    bottom: Val::Px(80.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                },
                visibility: Visibility::Hidden,
                ..default()
            },
            Interaction::None,
        ))
        .with_children(|p1| {
            tool_group(p1, "TORCHES", fnt2.clone(), |p2| {
                tool_btn(
                    p2,
                    "Torch",
                    Tool::SpawnStructure(BlueprintType::TorchStanding),
                    None,
                    fnt1.clone(),
                );
                tool_btn(
                    p2,
                    "Torch (wall)",
                    Tool::SpawnStructure(BlueprintType::TorchWall),
                    None,
                    fnt1.clone(),
                );
            });
            tool_group(p1, "OTHER", fnt2.clone(), |p2| {
                tool_btn(
                    p2,
                    "Workbench",
                    Tool::SpawnStructure(BlueprintType::Workbench),
                    None,
                    fnt1.clone(),
                );
                tool_btn(
                    p2,
                    "Ladder",
                    Tool::SpawnStructure(BlueprintType::Ladder),
                    None,
                    fnt1.clone(),
                );
                tool_btn(
                    p2,
                    "Door",
                    Tool::SpawnStructure(BlueprintType::Door),
                    None,
                    fnt1.clone(),
                );
            });
        })
        .id();

    toolbar.submenus.insert(SubmenuType::Blocks, block_submenu);
    toolbar
        .submenus
        .insert(SubmenuType::Structures, build_submenu);
}
