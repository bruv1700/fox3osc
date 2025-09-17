pub fn integrate_square_wave(p: f32, transition_size: f32) -> f32 {
    let mut value = 0.0;
    let mut prest = p;

    if p <= transition_size {
        value += self::integrate_f1(p / transition_size) * transition_size;
    } else {
        value += (2.0 / 3.0) * transition_size;
        prest -= transition_size;

        if p <= 0.5 - transition_size {
            value += prest;
        } else {
            value += 0.5 - 2.0 * transition_size;
            prest -= 0.5 - 2.0 * transition_size;

            if p <= 0.5 {
                value += ((2.0 / 3.0) - self::integrate_f1(1.0 - prest / transition_size))
                    * transition_size;
            } else {
                value += (2.0 / 3.0) * transition_size;
                prest -= transition_size;
                value -= self::integrate_square_wave(prest, transition_size);
            }
        }
    }

    value
}

pub fn integrate_f1(p: f32) -> f32 {
    -p.powf(3.0) / 3.0 + p.powf(2.0)
}

/// "Polynomial bandlimited step" algorithm. Smooths an aliased waveform at the transition points
/// using bandlimited polynomials.
pub fn polyblep(ptrans: f32) -> f32 {
    if ptrans <= -1.0 || ptrans >= 1.0 {
        0.0
    } else if ptrans <= 0.0 {
        (ptrans + 1.0).powf(2.0)
    } else {
        -(ptrans - 1.0).powf(2.0)
    }
}
