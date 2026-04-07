use bevy::prelude::*;
use bevy::render::view::RenderLayers;

use crate::data::{EquipSlot, EquipmentState, ITEMS};

#[derive(Component)]
pub struct Mannequin;

/// Stored render layer for propagation to scene children.
#[derive(Component)]
pub(crate) struct MannequinLayer(RenderLayers);

/// Key attachment point entities resolved from GLB named nodes.
#[derive(Resource)]
pub struct MannequinParts {
    pub head: Entity,
    pub torso: Entity,
    pub hand_r: Entity,
    pub hand_l: Entity,
    pub render_layer: RenderLayers,
}

/// Marker for spawned equipment visuals (despawned when unequipped).
#[derive(Component)]
pub struct EquipVisual(#[allow(dead_code)] pub EquipSlot);

pub fn spawn_mannequin(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pivot: Entity,
    render_layer: RenderLayers,
) {
    let scene = asset_server.load("models/mannequin.glb#Scene0");
    let mannequin = commands
        .spawn((
            SceneRoot(scene),
            Transform::from_translation(Vec3::new(0.0, -0.65, 0.0))
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            Mannequin,
            MannequinLayer(render_layer),
            Name::new("Mannequin"),
        ))
        .id();
    commands.entity(pivot).add_child(mannequin);
}

/// Resolves attachment points from GLB named nodes and propagates RenderLayers.
/// Runs until MannequinParts resource is inserted.
pub fn resolve_mannequin(
    mannequin_query: Query<&MannequinLayer, With<Mannequin>>,
    names: Query<(Entity, &Name)>,
    mesh_entities: Query<Entity, With<Mesh3d>>,
    mut commands: Commands,
) {
    let Ok(layer) = mannequin_query.single() else {
        return;
    };

    let mut head = None;
    let mut torso = None;
    let mut hand_r = None;
    let mut hand_l = None;

    for (entity, name) in &names {
        match name.as_str() {
            "attach_head" => head = Some(entity),
            "attach_torso" => torso = Some(entity),
            "attach_hand_r" => hand_r = Some(entity),
            "attach_hand_l" => hand_l = Some(entity),
            _ => {}
        }
    }

    let (Some(head), Some(torso), Some(hand_r), Some(hand_l)) =
        (head, torso, hand_r, hand_l)
    else {
        return; // Scene not loaded yet
    };

    // Propagate RenderLayers to all mesh entities in the scene
    for entity in &mesh_entities {
        commands.entity(entity).insert(layer.0.clone());
    }

    // Also add to attachment point entities
    for entity in [head, torso, hand_r, hand_l] {
        commands.entity(entity).insert(layer.0.clone());
    }

    commands.insert_resource(MannequinParts {
        head,
        torso,
        hand_r,
        hand_l,
        render_layer: layer.0.clone(),
    });
}

/// Propagates RenderLayers to all mesh descendants of EquipVisual and Mannequin scenes.
/// Runs every frame to catch newly loaded GLB children.
pub fn propagate_render_layers(
    parts: Res<MannequinParts>,
    mesh_entities: Query<(Entity, Option<&RenderLayers>), With<Mesh3d>>,
    parent_query: Query<&ChildOf>,
    mannequin_query: Query<Entity, With<Mannequin>>,
    equip_query: Query<Entity, With<EquipVisual>>,
    mut commands: Commands,
) {
    let Ok(mannequin) = mannequin_query.single() else {
        return;
    };

    // Collect root entities that should propagate render layers
    let mut roots: Vec<Entity> = vec![mannequin];
    roots.extend(equip_query.iter());

    for (mesh_entity, existing_layer) in &mesh_entities {
        if existing_layer.is_some() {
            continue; // Already has RenderLayers
        }
        // Walk up parents to check if this mesh is a descendant of a root
        let mut current = mesh_entity;
        loop {
            if roots.contains(&current) {
                commands.entity(mesh_entity).insert(parts.render_layer.clone());
                break;
            }
            if let Ok(child_of) = parent_query.get(current) {
                current = child_of.parent();
            } else {
                break;
            }
        }
    }
}

// ============================================================
// Equipment visual sync
// ============================================================

pub fn sync_mannequin_equipment(
    equip: Res<EquipmentState>,
    parts: Res<MannequinParts>,
    existing: Query<(Entity, &EquipVisual)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !equip.is_changed() {
        return;
    }

    // Remove old visuals
    for (entity, _) in &existing {
        commands.entity(entity).despawn();
    }

    // Head
    if let Some(idx) = equip.get(EquipSlot::Head) {
        let def = &ITEMS[idx];
        spawn_equip_visual(
            &mut commands,
            &asset_server,
            &parts,
            EquipSlot::Head,
            parts.head,
            def,
            &mut meshes,
            &mut materials,
        );
    }

    // Chest
    if let Some(idx) = equip.get(EquipSlot::Chest) {
        let def = &ITEMS[idx];
        spawn_equip_visual(
            &mut commands,
            &asset_server,
            &parts,
            EquipSlot::Chest,
            parts.torso,
            def,
            &mut meshes,
            &mut materials,
        );
    }

    // MainHand
    if let Some(idx) = equip.get(EquipSlot::MainHand) {
        let def = &ITEMS[idx];
        spawn_weapon_visual(
            &mut commands,
            &parts,
            parts.hand_r,
            def,
            &mut meshes,
            &mut materials,
        );
    }

    // OffHand
    if let Some(idx) = equip.get(EquipSlot::OffHand) {
        let def = &ITEMS[idx];
        spawn_offhand_visual(
            &mut commands,
            &parts,
            parts.hand_l,
            def,
            &mut meshes,
            &mut materials,
        );
    }
}

fn spawn_equip_visual(
    commands: &mut Commands,
    asset_server: &AssetServer,
    parts: &MannequinParts,
    slot: EquipSlot,
    parent: Entity,
    def: &crate::data::ItemDef,
    _meshes: &mut Assets<Mesh>,
    _materials: &mut Assets<StandardMaterial>,
) {
    if !def.model.is_empty() {
        // GLB model
        let scene = asset_server.load(format!("{}#Scene0", def.model));
        let visual = commands
            .spawn((
                SceneRoot(scene),
                Transform::default(),
                parts.render_layer.clone(),
                EquipVisual(slot),
            ))
            .id();
        commands.entity(parent).add_child(visual);
    }
}

fn spawn_weapon_visual(
    commands: &mut Commands,
    parts: &MannequinParts,
    parent: Entity,
    def: &crate::data::ItemDef,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    match def.name {
        "Iron Sword" => {
            let mesh = meshes.add(Cuboid::new(0.03, 0.4, 0.02));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.6, 0.6, 0.65),
                metallic: 0.8,
                perceptual_roughness: 0.2,
                ..default()
            });
            let e = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(0.0, 0.22, 0.0),
                    parts.render_layer.clone(),
                    EquipVisual(EquipSlot::MainHand),
                ))
                .id();
            commands.entity(parent).add_child(e);
        }
        "Battle Axe" => {
            // Handle
            let handle_mesh = meshes.add(Cuboid::new(0.025, 0.35, 0.025));
            let handle_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.28, 0.15),
                perceptual_roughness: 0.8,
                ..default()
            });
            let handle = commands
                .spawn((
                    Mesh3d(handle_mesh),
                    MeshMaterial3d(handle_mat),
                    Transform::from_xyz(0.0, 0.175, 0.0),
                    parts.render_layer.clone(),
                    EquipVisual(EquipSlot::MainHand),
                ))
                .id();
            commands.entity(parent).add_child(handle);
            // Axe head
            let head_mesh = meshes.add(Cuboid::new(0.15, 0.10, 0.03));
            let head_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.48, 0.5),
                metallic: 0.7,
                perceptual_roughness: 0.3,
                ..default()
            });
            let head = commands
                .spawn((
                    Mesh3d(head_mesh),
                    MeshMaterial3d(head_mat),
                    Transform::from_xyz(0.06, 0.32, 0.0),
                    parts.render_layer.clone(),
                    EquipVisual(EquipSlot::MainHand),
                ))
                .id();
            commands.entity(parent).add_child(head);
        }
        "Arcane Staff" => {
            // Long staff
            let mesh = meshes.add(Cylinder::new(0.02, 0.49));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.2, 0.4),
                perceptual_roughness: 0.6,
                ..default()
            });
            let staff = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(0.0, 0.245, 0.0),
                    parts.render_layer.clone(),
                    EquipVisual(EquipSlot::MainHand),
                ))
                .id();
            commands.entity(parent).add_child(staff);
            // Orb on top
            let orb_mesh = meshes.add(Sphere::new(0.045));
            let orb_mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.5, 0.3, 0.8),
                emissive: bevy::color::LinearRgba::new(0.8, 0.4, 1.5, 1.0),
                ..default()
            });
            let orb = commands
                .spawn((
                    Mesh3d(orb_mesh),
                    MeshMaterial3d(orb_mat),
                    Transform::from_xyz(0.0, 0.50, 0.0),
                    parts.render_layer.clone(),
                    EquipVisual(EquipSlot::MainHand),
                ))
                .id();
            commands.entity(parent).add_child(orb);
        }
        _ => {}
    }
}

fn spawn_offhand_visual(
    commands: &mut Commands,
    parts: &MannequinParts,
    parent: Entity,
    def: &crate::data::ItemDef,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    match def.name {
        "Wooden Shield" => {
            let mesh = meshes.add(Cylinder::new(0.12, 0.02));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.3, 0.2),
                perceptual_roughness: 0.7,
                ..default()
            });
            let e = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(0.0, 0.0, -0.08)
                        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    parts.render_layer.clone(),
                    EquipVisual(EquipSlot::OffHand),
                ))
                .id();
            commands.entity(parent).add_child(e);
        }
        "Tome of Lore" => {
            let mesh = meshes.add(Cuboid::new(0.1, 0.14, 0.03));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.35, 0.2, 0.5),
                perceptual_roughness: 0.7,
                ..default()
            });
            let e = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::from_xyz(0.0, 0.0, -0.06),
                    parts.render_layer.clone(),
                    EquipVisual(EquipSlot::OffHand),
                ))
                .id();
            commands.entity(parent).add_child(e);
        }
        _ => {}
    }
}
