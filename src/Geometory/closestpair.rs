// Source: https://github.com/InverseAki/arcaki_library/blob/main/src/Geometory/closestpair.rs
pub fn closest_pair<T: GeometryCoordinate>(points: &[Point<T>]) -> Option<f64> {
    closest_pair_squared(points).map(|distance| distance.rational().to_f64().sqrt())
}

pub fn closest_pair_squared<T: GeometryCoordinate>(points: &[Point<T>]) -> Option<T::Wide> {
    for p in points {
        p.wide();
    }
    if points.len() < 2 {
        return None;
    }
    let mut points = points.to_vec();
    points.sort_unstable();
    let mut scratch = points.clone();
    Some(closest_pair_find(&mut points, &mut scratch))
}

fn closest_pair_find<T: GeometryCoordinate>(
    points: &mut [Point<T>],
    scratch: &mut [Point<T>],
) -> T::Wide {
    if points.len() <= 3 {
        let mut best = points[0].distance_squared(points[1]);
        for i in 0..points.len() {
            for j in i + 1..points.len() {
                best = best.min(points[i].distance_squared(points[j]));
            }
        }
        points.sort_unstable_by_key(|p| (p.y, p.x));
        return best;
    }
    let mid = points.len() / 2;
    let center = points[mid].x.wide();
    let (left, right) = points.split_at_mut(mid);
    let (left_scratch, right_scratch) = scratch.split_at_mut(mid);
    let mut best =
        closest_pair_find(left, left_scratch).min(closest_pair_find(right, right_scratch));
    let (mut i, mut j) = (0, mid);
    for p in scratch.iter_mut() {
        if i < mid
            && (j == points.len() || (points[i].y, points[i].x) <= (points[j].y, points[j].x))
        {
            *p = points[i];
            i += 1;
        } else {
            *p = points[j];
            j += 1;
        }
    }
    points.copy_from_slice(scratch);
    if best == T::Wide::zero() {
        return best;
    }
    let mut len = 0;
    for &p in points.iter() {
        let dx = p.x.wide().sub(center);
        if dx.mul(dx) >= best {
            continue;
        }
        for &q in scratch[..len].iter().rev() {
            let dy = p.y.wide().sub(q.y.wide());
            if dy.mul(dy) >= best {
                break;
            }
            best = best.min(p.distance_squared(q));
        }
        if best == T::Wide::zero() {
            return best;
        }
        scratch[len] = p;
        len += 1;
    }
    best
}
