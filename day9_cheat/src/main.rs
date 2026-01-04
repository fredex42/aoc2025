use std::fs::File;
use std::io::{self, Read};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

fn main() -> io::Result<()> {
    // Read input file “09.txt”
    let mut contents = String::new();
    File::open("day9.txt")?.read_to_string(&mut contents)?;
    let mut points: Vec<Point> = Vec::new();

    for line in contents.lines() {
        let parts: Vec<_> = line.split(',').collect();
        if parts.len() == 2 {
            let x: i32 = parts[0].trim().parse().unwrap();
            let y: i32 = parts[1].trim().parse().unwrap();
            points.push(Point { x, y });
        }
    }

    // Group by x and by y
    let mut by_x: HashMap<i32, Vec<i32>> = HashMap::new();
    let mut by_y: HashMap<i32, Vec<i32>> = HashMap::new();
    for p in &points {
        by_x.entry(p.x).or_default().push(p.y);
        by_y.entry(p.y).or_default().push(p.x);
    }

    // Build segments
    let mut h_segs: Vec<(Point, Point)> = Vec::new();
    for (&x, ys) in &by_x {
        let mut sorted = ys.clone();
        sorted.sort_unstable();
        for chunk in sorted.chunks(2) {
            if chunk.len() == 2 {
                let a = Point { x, y: chunk[0] };
                let b = Point { x, y: chunk[1] };
                h_segs.push((a, b));
            }
        }
    }

    let mut v_segs: Vec<(Point, Point)> = Vec::new();
    for (&y, xs) in &by_y {
        let mut sorted = xs.clone();
        sorted.sort_unstable();
        for chunk in sorted.chunks(2) {
            if chunk.len() == 2 {
                let a = Point { x: chunk[0], y };
                let b = Point { x: chunk[1], y };
                v_segs.push((a, b));
            }
        }
    }

    // Search for best rectangles
    let mut maxa: i64 = 0;
    let mut maxb: i64 = 0;
    let mut best_rect: Option<(i32, i32, i32, i32)> = None;

    for i in 0..points.len() {
        for j in 0..i {
            let a = points[i];
            let b = points[j];
            let (minx, maxx) = (a.x.min(b.x), a.x.max(b.x));
            let (miny, maxy) = (a.y.min(b.y), a.y.max(b.y));

            let area = ((maxx - minx + 1) as i64) * ((maxy - miny + 1) as i64);
            maxa = maxa.max(area);

            // Check if rect intersects any segments
            let mut works = true;

            for (h0, h1) in &h_segs {
                let hx = h0.x;
                let hy_min = h0.y.min(h1.y);
                let hy_max = h0.y.max(h1.y);

                if hx > minx && hx < maxx {
                    let ok = hy_max <= miny || hy_min >= maxy;
                    if !ok {
                        works = false;
                        break;
                    }
                }
            }

            if !works {
                continue;
            }

            for (v0, v1) in &v_segs {
                let vy = v0.y;
                let vx_min = v0.x.min(v1.x);
                let vx_max = v0.x.max(v1.x);

                if vy > miny && vy < maxy {
                    let ok = vx_max <= minx || vx_min >= maxx;
                    if !ok {
                        works = false;
                        break;
                    }
                }
            }

            if works && area > maxb {
                maxb = area;
                best_rect = Some((minx, miny, maxx, maxy));
            }
        }
    }

    println!("max possible area = {}", maxa);
    println!("max valid area = {}", maxb);

    if let Some((bx0, by0, bx1, by1)) = best_rect {
        println!("best rect: ({},{})->({}, {})", bx0, by0, bx1, by1);
    }

    Ok(())
}
