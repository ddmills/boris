/// Flood fill starting from `seed` location. Every point is checked
/// against `fill`. The `fill` function needs to both check if the point
/// should be filled (bool), and fill it in.
#[allow(dead_code)]
pub fn flood_fill_i32<F: FnMut([i32; 3]) -> bool>(seed: [i32; 3], mut fill: F) {
    let mut queue = vec![seed];

    while let Some(p) = queue.pop() {
        if fill(p) {
            queue.push([p[0] + 1, p[1], p[2]]);
            queue.push([p[0] - 1, p[1], p[2]]);
            queue.push([p[0], p[1] + 1, p[2]]);
            queue.push([p[0], p[1] - 1, p[2]]);
            queue.push([p[0], p[1], p[2] + 1]);
            queue.push([p[0], p[1], p[2] - 1]);
        }
    }
}

#[allow(dead_code)]
pub fn flood_fill<T: Copy, F: FnMut(T) -> bool, N: FnMut(T) -> Vec<T>>(
    seed: T,
    mut fill: F,
    mut neighbors: N,
) {
    let mut queue = vec![seed];

    while let Some(p) = queue.pop() {
        if fill(p) {
            for neighbor in neighbors(p).iter() {
                queue.push(*neighbor);
            }
        }
    }
}
