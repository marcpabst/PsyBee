// Copyright (c) 2024 Marc Pabst
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
use half::f16;
use plotters::prelude::*;

/// Function that converts a sRGB float to linear
/// as per ITU-R BT.709
fn srgb_to_linear(x: f32) -> f32 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

fn main() {
    // count all floats between 0 and 1
    let mut count = 0;
    let mut srgb = Vec::new();
    let mut linear = Vec::new();
    for i in 0..u16::MAX {
        let f = f16::from_bits(i);
        if f.to_f32() > 0.0 && f.to_f32() < 1.0 {
            linear.push(srgb_to_linear(f.to_f32()));
            srgb.push(f.to_f32());
            count += 1;
        } else if f.to_f32() >= 1.0 {
            break;
        }
    }

    // let's get all u8 values 0..255 as floats between 0 and 1
    let mut srgb_u8 = Vec::new();
    let mut linear_u8 = Vec::new();
    for i in 0..u8::MAX {
        let f = i as f32 / 255.0;
        linear_u8.push(srgb_to_linear(f));
        srgb_u8.push(f);
    }

    // plot the floats
    let root = SVGBackend::new("srgb_to_linear.svg", (1000, 700)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(0.0..1.0, 0.0..1.0)
        .unwrap();

    chart
        .configure_mesh()
        .x_labels(10)
        .y_labels(10)
        .x_desc("Physical Luminance")
        .x_label_style(("sans-serif", 25).into_font())
        .y_desc("sRGB Luminance")
        .y_label_style(("sans-serif", 25).into_font())
        .draw()
        .unwrap();

    // draw the sRGB chart
    chart
        .draw_series(LineSeries::new(
            linear
                .iter()
                .zip(srgb.iter())
                .map(|(x, y)| (*x as f64, *y as f64)),
            &RED,
        ))
        .unwrap();

    // draw scatter at y=0
    chart
        .draw_series(PointSeries::of_element(
            linear_u8
                .iter()
                .zip(srgb_u8.iter())
                .map(|(x, y)| (*x as f64, 0.0)),
            5,
            &BLACK,
            &|c, s, st| {
                return EmptyElement::at(c)
                    + Rectangle::new([(0, 0), (1, -10)], st.filled());
            },
        ))
        .unwrap();

    // draw scatter at x=0
    chart
        .draw_series(PointSeries::of_element(
            linear_u8
                .iter()
                .zip(srgb_u8.iter())
                .map(|(x, y)| (0.0, *y as f64)),
            5,
            &BLACK,
            &|c, s, st| {
                return EmptyElement::at(c)
                    + Rectangle::new([(0, 0), (10, 1)], st.filled());
            },
        ))
        .unwrap();

    // save the plot
    root.present().unwrap();
}

/// Function that plots the sRGB color space in x,y coordinates
fn plot_srgb() {
    
}
