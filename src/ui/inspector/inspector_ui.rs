use bevy::{
    asset::{AssetServer, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        event::{Event, EventReader},
        query::{Changed, With, Without},
        system::{Commands, Query, Res, ResMut, Resource},
    },
    hierarchy::BuildChildren,
    prelude::default,
    render::{color::Color, mesh::Mesh, view::Visibility},
    text::{Text, TextStyle},
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        AlignContent, AlignItems, BackgroundColor, Display, FlexDirection, FocusPolicy,
        Interaction, JustifyContent, PositionType, Style, UiRect, Val,
    },
};
use bevy_mod_picking::{
    backends::raycast::bevy_mod_raycast::markers::SimplifiedMesh,
    events::{Down, Pointer},
    picking_core::Pickable,
    prelude::{ListenerInput, On},
    PickableBundle,
};

use crate::{
    items::{Commodities, Commodity, CommodityData},
    rendering::SlotIndex,
    structures::PartSlots,
    ui::{BTN_HOVERED, BTN_NONE, BTN_PRESSED, BTN_TOGGLED},
};

#[derive(Resource)]
pub struct InspectorUi {
    pub selected: Option<Entity>,
    pub window: Entity,
    pub display_text: Entity,
    pub slots: Entity,
    pub slot_0: Entity,
    pub slot_1: Entity,
    pub slot_2: Entity,
}

#[derive(Event)]
pub struct InspectableClickedEvent(Entity, f32);

impl From<ListenerInput<Pointer<Down>>> for InspectableClickedEvent {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        InspectableClickedEvent(event.target, event.hit.depth)
    }
}

pub fn update_inspector(
    mut cmd: Commands,
    inspector: ResMut<InspectorUi>,
    q_inspectables: Query<&Inspectable>,
    q_slots: Query<&PartSlots>,
    q_commodities: Query<&Commodity>,
    commodities: Res<Commodities>,
    mut q_text: Query<&mut Text>,
) {
    let Some(inspectable_e) = inspector.selected else {
        return;
    };

    let Ok(inspectable) = q_inspectables.get(inspectable_e) else {
        return;
    };

    let Ok(mut display_text) = q_text.get_mut(inspector.display_text) else {
        return;
    };

    display_text
        .sections
        .get_mut(0)
        .unwrap()
        .value
        .clone_from(&inspectable.display_name);

    if let Ok(slots) = q_slots.get(inspectable_e) {
        if let Some(commodity) = get_commodity_data(slots, SlotIndex::Slot0, &q_commodities) {
            let commodity_data = commodities.0.get(&commodity).unwrap();
            let mut slot_txt = q_text.get_mut(inspector.slot_0).unwrap();

            slot_txt.sections[0].value.clone_from(&commodity_data.name);
            cmd.entity(inspector.slot_0).insert(Visibility::Inherited);
        } else {
            cmd.entity(inspector.slot_0).insert(Visibility::Hidden);
        };

        if let Some(commodity) = get_commodity_data(slots, SlotIndex::Slot1, &q_commodities) {
            let commodity_data = commodities.0.get(&commodity).unwrap();
            let mut slot_txt = q_text.get_mut(inspector.slot_1).unwrap();

            slot_txt.sections[0].value.clone_from(&commodity_data.name);
            cmd.entity(inspector.slot_1).insert(Visibility::Inherited);
        } else {
            cmd.entity(inspector.slot_1).insert(Visibility::Hidden);
        };

        if let Some(commodity) = get_commodity_data(slots, SlotIndex::Slot2, &q_commodities) {
            let commodity_data = commodities.0.get(&commodity).unwrap();
            let mut slot_txt = q_text.get_mut(inspector.slot_2).unwrap();

            slot_txt.sections[0].value.clone_from(&commodity_data.name);
            cmd.entity(inspector.slot_2).insert(Visibility::Inherited);
        } else {
            cmd.entity(inspector.slot_2).insert(Visibility::Hidden);
        };
        cmd.entity(inspector.slots).insert(Visibility::Inherited);
    } else {
        cmd.entity(inspector.slots).insert(Visibility::Hidden);
    }
}

fn get_commodity_data(
    slots: &PartSlots,
    slot_idx: SlotIndex,
    q_commodities: &Query<&Commodity>,
) -> Option<Commodity> {
    let slot = slots.get(slot_idx)?;
    let content = slot.content?;
    let Ok(commodity) = q_commodities.get(content) else {
        return None;
    };
    Some(*commodity)
}

pub fn on_inspectable_clicked(
    mut cmd: Commands,
    mut ev_inspectable_clicked: EventReader<InspectableClickedEvent>,
    mut inspector: ResMut<InspectorUi>,
    q_inspectables: Query<&Inspectable>,
) {
    for ev in ev_inspectable_clicked.read() {
        if !q_inspectables.contains(ev.0) {
            println!("Non-inspectable?");
            continue;
        };

        inspector.selected = Some(ev.0);
        cmd.entity(inspector.window).insert(Visibility::Inherited);
    }
}

#[derive(Component)]
pub struct BtnInspectorClose;

#[derive(Component)]
pub struct Inspectable {
    pub display_name: String,
}

pub fn setup_inspector_ui(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let fnt1 = asset_server.load("fonts/Yantramanav/Yantramanav-Black.ttf");
    let fnt2 = asset_server.load("fonts/Averia_Serif/AveriaSerifLibre-Regular.ttf");

    let mut display_text = None;
    let mut slots = None;
    let mut slot_1 = None;
    let mut slot_2 = None;
    let mut slot_3 = None;

    let inspector = cmd
        .spawn(NodeBundle {
            focus_policy: FocusPolicy::Block,
            style: Style {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.),
                display: Display::Flex,
                position_type: PositionType::Absolute,
                width: Val::Px(300.),
                height: Val::Px(400.),
                top: Val::Px(24.),
                left: Val::Px(24.),
                border: UiRect::all(Val::Px(2.0)),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            background_color: Color::rgba(0.2, 0.2, 0.2, 0.4).into(),
            border_color: Color::rgb(0.1, 0.1, 0.1).into(),
            visibility: Visibility::Hidden,
            ..default()
        })
        .with_children(|p1| {
            display_text = Some(
                p1.spawn(TextBundle {
                    text: Text::from_section(
                        "Unknown",
                        TextStyle {
                            font_size: 24.0,
                            font: fnt2,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    ..default()
                })
                .id(),
            );

            p1.spawn((
                ButtonBundle {
                    style: Style {
                        padding: UiRect::axes(Val::Px(16.), Val::Px(8.)),
                        height: Val::Px(32.),
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
                BtnInspectorClose,
            ))
            .with_children(|p2| {
                p2.spawn(TextBundle {
                    text: Text::from_section(
                        "Close",
                        TextStyle {
                            font_size: 18.0,
                            font: fnt1.clone(),
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ),
                    ..default()
                });
            });

            slots = Some(
                p1.spawn(NodeBundle {
                    style: Style {
                        // padding: UiRect::axes(Val::Px(16.), Val::Px(8.)),
                        // height: Val::Px(32.),
                        // justify_content: JustifyContent::Center,
                        // align_content: AlignContent::Center,
                        // align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.),
                        display: Display::Flex,
                        ..default()
                    },
                    background_color: Color::OLIVE.into(),
                    ..default()
                })
                .with_children(|p2| {
                    slot_1 = Some(
                        p2.spawn(TextBundle {
                            text: Text::from_section(
                                "Slot 1",
                                TextStyle {
                                    font_size: 18.0,
                                    font: fnt1.clone(),
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            ),
                            ..default()
                        })
                        .id(),
                    );
                    slot_2 = Some(
                        p2.spawn(TextBundle {
                            text: Text::from_section(
                                "Slot 2",
                                TextStyle {
                                    font_size: 18.0,
                                    font: fnt1.clone(),
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            ),
                            ..default()
                        })
                        .id(),
                    );
                    slot_3 = Some(
                        p2.spawn(TextBundle {
                            text: Text::from_section(
                                "Slot 3",
                                TextStyle {
                                    font_size: 18.0,
                                    font: fnt1.clone(),
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            ),
                            ..default()
                        })
                        .id(),
                    );
                })
                .id(),
            );
        })
        .id();

    cmd.insert_resource(InspectorUi {
        selected: None,
        window: inspector,
        display_text: display_text.unwrap(),
        slots: slots.unwrap(),
        slot_0: slot_1.unwrap(),
        slot_1: slot_2.unwrap(),
        slot_2: slot_3.unwrap(),
    });
}

pub fn on_inspector_close(
    mut cmd: Commands,
    mut inspector: ResMut<InspectorUi>,
    mut btn_query: Query<
        (&Interaction, &mut BackgroundColor),
        (With<BtnInspectorClose>, Changed<Interaction>),
    >,
) {
    for (interaction, mut bkg) in &mut btn_query {
        match *interaction {
            Interaction::Pressed => {
                bkg.0 = BTN_PRESSED;
                cmd.entity(inspector.window).insert(Visibility::Hidden);
                inspector.selected = None;
            }
            Interaction::Hovered => {
                bkg.0 = BTN_HOVERED;
            }
            Interaction::None => {
                bkg.0 = BTN_NONE;
            }
        }
    }
}

pub fn setup_inspectables(
    mut cmd: Commands,
    q_inspectables: Query<Entity, (With<Handle<Mesh>>, With<Inspectable>, Without<Pickable>)>,
) {
    for entity in q_inspectables.iter() {
        cmd.entity(entity)
            .try_insert(PickableBundle::default())
            .try_insert(On::<Pointer<Down>>::send_event::<InspectableClickedEvent>());
    }
}
