pub fn running_as_root() -> bool {
    use users::{get_current_uid, get_user_by_uid};

    let user = get_user_by_uid(get_current_uid()).unwrap();
    match user.name().to_str() {
        Some("root") => true,
        _ => false,
    }
}
