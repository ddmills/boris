/// Flood fill starting from `seed` location. Every point is checked
/// against `fill`. The `fill` function needs to both check if the point
/// should be filled (bool), and fill it in.
#[allow(dead_code)]
pub fn flood_fill<F: Fn([i32; 3]) -> bool>(seed: [i32; 3], fill: F) {
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
