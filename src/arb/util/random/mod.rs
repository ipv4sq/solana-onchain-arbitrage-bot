use rand::Rng;

pub fn random_select<T>(items: &[T]) -> Option<&T> {
    if items.is_empty() {
        None
    } else {
        let index = rand::rng().random_range(0..items.len());
        Some(&items[index])
    }
}
