use bevy::{prelude::*, ui::FocusPolicy};

use crate::Block;

use super::Tool;

const BTN_PRESSED: Color = Color::BLUE;
const BTN_NONE: Color = Color::ORANGE_RED;
const BTN_HOVERED: Color = Color::GREEN;
const BTN_TOGGLED: Color = Color::PURPLE;

#[derive(Component)]
pub struct BtnToolbarBlock {
    block: Block,
}

#[derive(Resource)]
pub struct Toolbar {
    pub tool: Tool,
}

pub fn toolbar_select(
    mut toolbar: ResMut<Toolbar>,
    mut btn_query: Query<(&Interaction, &BtnToolbarBlock, &mut BackgroundColor)>,
) {
    for (interaction, btn, mut bkg) in &mut btn_query {
        match *interaction {
            Interaction::Pressed => {
                toolbar.tool = Tool::PlaceBlocks(btn.block);
                bkg.0 = BTN_PRESSED;
            }
            Interaction::Hovered => {
                bkg.0 = BTN_HOVERED;
            }
            Interaction::None => {
                bkg.0 = BTN_NONE;
            }
        }

        match toolbar.tool {
            Tool::PlaceBlocks(block) => {
                if btn.block == block {
                    bkg.0 = BTN_TOGGLED;
                }
            }
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
            vec![
                Block::GRASS,
                Block::DIRT,
                Block::STONE,
                Block::LAMP,
                Block::MAGMA,
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
                        BtnToolbarBlock { block },
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
