pub fn kallyup_profile_components(profile: u32) -> u32 {
    if (profile == 1) {
        return 3;
    }
    if (profile == 2) {
        return 7;
    }
    if (profile == 3) {
        return 15;
    }
    return 0;
}

pub fn kallyup_profile_valid(profile: u32) -> bool {
    return ((profile >= 1) && (profile <= 3));
}

pub fn kallyup_component_enabled(components: u32, component: u32) -> bool {
    return ((components & component) != 0);
}

