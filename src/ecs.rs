type ComponentId = u32;

struct Entity {
    id: u32,
    components: Vec<ComponentId>,
}

impl Entity {
    fn has_component(&self, component_id: &ComponentId) -> bool {
        self.components.contains(component_id)
    }
}

// component id 1
struct Vec2 {
    x: f32,
    y: f32,
}

// run for every struct with the Vec component
fn run() {

}

fn update_positions(entities: Vec<Entity>, position_component: ComponentId, positions_resource: &Vec<Vec2>) {
    // let entites_with_position: Vec<Entity> = entities.iter().filter_map(|x| if x.has_component(&position_component) {Some(x)} else {None} ).collect();

    // for entity in entites_with_position {
    //
    // }
}

fn update_position(positions_resource: Vec<Vec2>) {

}
