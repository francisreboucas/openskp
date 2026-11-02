use openskp::create::{create, ArcOptions, CircleOptions, FaceOptions, PolylineOptions};
use openskp::internal::bytes::parse_tlv_recursive;

fn make_nested_tlv(levels: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(levels * 6);
    for i in (1..=levels).rev() {
        data.push(0xF4);
        data.push(0x01);
        let child_len = ((i - 1) * 6) as u32;
        data.extend_from_slice(&child_len.to_le_bytes());
    }
    data
}

#[test]
fn tlv_recursion_boundary_test() {
    // MAX_TLV_DEPTH is 64.
    // In make_nested_tlv: level 1 is at depth 0, level 2 is at depth 1, ..., level 65 is at depth 64.
    // Depth 64 satisfies depth <= MAX_TLV_DEPTH (64), so 65 levels must parse successfully.
    let data_65 = make_nested_tlv(65);
    assert!(
        parse_tlv_recursive(&data_65, 0, data_65.len()).is_ok(),
        "Nesting up to depth 64 (65 levels) must parse successfully"
    );

    // 66 levels attempts to recurse to depth 65 (> MAX_TLV_DEPTH 64), which must be rejected cleanly.
    let data_66 = make_nested_tlv(66);
    let result_66 = parse_tlv_recursive(&data_66, 0, data_66.len());
    assert!(
        result_66.is_err(),
        "Nesting exceeding MAX_TLV_DEPTH (depth 65) must be rejected"
    );
}

#[test]
fn add_face_rejects_nan_coordinates() {
    let mut builder = create();
    let pts = [
        [0.0, 0.0, 0.0],
        [f64::NAN, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
    ];
    let result = builder.add_face(&pts, &FaceOptions::default());
    assert!(
        result.is_err(),
        "add_face must reject NaN coordinates in outer boundary"
    );
}

#[test]
fn add_face_rejects_nan_in_holes() {
    let mut builder = create();
    let face_pts = [
        [0.0, 0.0, 0.0],
        [100.0, 0.0, 0.0],
        [100.0, 100.0, 0.0],
        [0.0, 100.0, 0.0],
    ];
    let hole_with_nan = vec![[10.0, 10.0, 0.0], [f64::NAN, 10.0, 0.0], [20.0, 20.0, 0.0]];
    let opts = FaceOptions {
        holes: vec![hole_with_nan],
        ..Default::default()
    };
    let result = builder.add_face(&face_pts, &opts);
    assert!(
        result.is_err(),
        "add_face must reject NaN coordinates inside holes"
    );
}

#[test]
fn add_circle_rejects_non_finite_center_and_normal() {
    let mut builder = create();
    // Non-finite center
    assert!(
        builder
            .add_circle(
                [f64::NAN, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                10.0,
                &CircleOptions::default(),
            )
            .is_err(),
        "add_circle must reject NaN center"
    );

    // Non-finite normal
    assert!(
        builder
            .add_circle(
                [0.0, 0.0, 0.0],
                [0.0, f64::INFINITY, 1.0],
                10.0,
                &CircleOptions::default(),
            )
            .is_err(),
        "add_circle must reject Infinity in normal"
    );

    // Non-finite radius
    assert!(
        builder
            .add_circle(
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                f64::NAN,
                &CircleOptions::default(),
            )
            .is_err(),
        "add_circle must reject NaN radius"
    );
}

#[test]
fn add_circle_allows_negative_radius_for_cpp_parity() {
    let mut builder = create();
    // Negative radius is allowed in C++ implementation, must not be rejected here
    let result = builder.add_circle(
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        -10.0,
        &CircleOptions::default(),
    );
    assert!(
        result.is_ok(),
        "add_circle must allow negative radius to maintain parity with C++ writer"
    );
}

#[test]
fn builders_reject_non_finite_coordinates() {
    let mut builder = create();

    // add_arc
    assert!(
        builder
            .add_arc(
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                f64::NAN,
                0.0,
                1.0,
                &ArcOptions::default(),
            )
            .is_err(),
        "add_arc must reject NaN radius"
    );
    assert!(
        builder
            .add_arc(
                [f64::NAN, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                10.0,
                0.0,
                1.0,
                &ArcOptions::default(),
            )
            .is_err(),
        "add_arc must reject NaN center"
    );

    // add_polyline
    assert!(
        builder
            .add_polyline(
                &[[0.0, 0.0, 0.0], [f64::NAN, 10.0, 0.0]],
                &PolylineOptions::default(),
            )
            .is_err(),
        "add_polyline must reject NaN point"
    );

    // add_construction_point
    assert!(
        builder
            .add_construction_point([0.0, f64::INFINITY, 0.0])
            .is_err(),
        "add_construction_point must reject Infinity"
    );

    // add_dimension
    assert!(
        builder
            .add_dimension([0.0, 0.0, 0.0], [f64::NAN, 0.0, 0.0], 1.0)
            .is_err(),
        "add_dimension must reject NaN point"
    );
}
