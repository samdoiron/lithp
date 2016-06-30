pub fn prepend<T>(item: T, items: &mut Vec<T>) -> Vec<T> {
    let mut new = vec![item];
    new.append(items);
    new
}