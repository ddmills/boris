pub fn min_3(a: i32, b: i32, c: i32) -> i32 {
    if a <= b && a <= c {
        return a;
    }

    if b <= a && b <= c {
        return b;
    }

    c
}

pub fn max_3(a: i32, b: i32, c: i32) -> i32 {
    if a >= b && a >= c {
        return a;
    }

    if b >= a && b >= c {
        return b;
    }

    c
}

pub fn sig_num(v: f32) -> i32 {
    if v > 0. {
        1
    } else if v < 0. {
        -1
    } else {
        0
    }
}

pub fn min_max(a: u32, b: u32) -> [u32; 2] {
    if a > b {
        [b, a]
    } else {
        [a, b]
    }
}
