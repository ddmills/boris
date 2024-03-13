#[allow(dead_code)]
pub fn min_3(a: i32, b: i32, c: i32) -> i32 {
    if a <= b && a <= c {
        return a;
    }

    if b <= a && b <= c {
        return b;
    }

    c
}

#[allow(dead_code)]
pub fn max_3(a: i32, b: i32, c: i32) -> i32 {
    if a >= b && a >= c {
        return a;
    }

    if b >= a && b >= c {
        return b;
    }

    c
}

#[allow(dead_code)]
pub fn sig_num(v: f32) -> i32 {
    if v > 0. {
        1
    } else if v < 0. {
        -1
    } else {
        0
    }
}

#[allow(dead_code)]
pub fn min_max(a: u32, b: u32) -> [u32; 2] {
    if a > b {
        [b, a]
    } else {
        [a, b]
    }
}

#[allow(dead_code)]
pub fn min_max_3<T>(a: T, b: T, c: T) -> [T; 3]
where
    T: std::cmp::PartialOrd,
{
    if a > b {
        if a > c {
            if b > c {
                return [c, b, a];
            }

            return [b, c, a];
        }

        return [b, a, c];
    }
    if b > c {
        if a > c {
            return [c, a, b];
        }
        return [a, c, b];
    }
    [a, b, c]
}
