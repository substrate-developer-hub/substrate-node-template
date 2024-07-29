#![allow(clippy::indexing_slicing)]

use frame_metadata::RuntimeMetadata;
use node_subtensor_runtime::Runtime;
use scale_info::TypeDef;

fn is_pallet_error(segments: &[String]) -> bool {
    let pallet_list: Vec<&str> = vec![
        "pallet_admin_utils",
        "pallet_collective",
        "pallet_commitments",
        "pallet_registry",
        "pallet_subtensor",
    ];

    if segments.len() != 3 {
        false
    } else {
        pallet_list.contains(&segments[0].as_str())
            && segments[1] == "pallet"
            && segments[2] == "Error"
    }
}

// test make sure all errors are documented
#[test]
fn test_metadata() {
    let metadata = Runtime::metadata().1;
    // current metadata version should be 14
    assert!(matches!(metadata, RuntimeMetadata::V14(_)));

    if let RuntimeMetadata::V14(value) = metadata {
        let types = value.types.types;
        for ty in types.iter() {
            let segments = &ty.ty.path.segments;
            if is_pallet_error(segments) {
                // error call and event should be enum type
                assert!(matches!(ty.ty.type_def, TypeDef::Variant(_)));
                if let TypeDef::Variant(variants) = &ty.ty.type_def {
                    // check docs not empty
                    for variant in variants.variants.iter() {
                        // print name make it easier to find out failed item
                        println!("{}", variant.name);
                        assert!(!variant.docs.is_empty());
                        assert!(!variant.docs[0].is_empty());
                    }
                }
            }
        }
    };
}
