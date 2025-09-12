use rand::Rng;

pub fn random_select<T>(items: &[T]) -> Option<&T> {
    if items.is_empty() {
        None
    } else {
        let index = rand::rng().random_range(0..items.len());
        Some(&items[index])
    }
}

pub fn random_choose<T>(items: &[T]) -> &T {
    items
        .get(rand::rng().random_range(0..items.len()))
        .expect("random_select_unwrap called with empty slice")
}
